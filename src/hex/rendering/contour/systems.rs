use bevy::asset::RenderAssetUsages;

use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use hexx::*;
use image::{DynamicImage, GenericImageView, Rgba};

use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

use crate::{
    hex::{
        HexCoord,
        rendering::contour::{ContourConfig, ContourPath},
    },
    shared::types::{CellData, TerrainType, Triangle, TriangleId},
};

pub fn setup_organic_contour(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // images: Res<Assets<Image>>,
    // asset_server: Res<AssetServer>,
) {
    // let binary_map_handle = asset_server.load("maps/Gaulyia_binarymap.png");

    let image_path = "assets/maps/Gaulyia_binarymap.png";
    let binary_map = image::open(image_path).expect("Failed to load binary map image");
    // Charger votre binary map
    // let binary_map = images.get(&binary_map).unwrap();

    // Configuration
    let config = ContourConfig {
        pixels_per_hex: 0.1, // 4 hex par pixel
        noise_amplitude: 0.2, // Force du bruit
        noise_frequency: 3.0, // Fréquence du bruit
        noise_octaves: 8,     // Détail fractal
        threshold: 0.5,       // Seuil terre/mer
        spline_tension: 0.8,  // Courbure des splines
    };

    // Créer le système
    let layout = HexLayout::flat().with_hex_size(48.0);
    let mut system = OrganicContourSystem::new(binary_map, layout, config);

    // Initialiser la grille
    system.initialize_hex_grid(100);

    // Générer les contours
    let contours = system.generate_organic_contours();

    // Créer le mesh
    let mesh = system.generate_mesh(&contours);
    info!("Mesh vertices count: {}", mesh.count_vertices());

    // Spawner l'entité
    commands.spawn((
        Name::new("Contour"),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.6, 0.4)))),
    ));
}

// Système principal de génération des contours
pub struct OrganicContourSystem {
    binary_map: DynamicImage,
    hex_layout: HexLayout,
    config: ContourConfig,
    hex_cells: HashMap<Hex, CellData>,
    perlin: Perlin,
}

impl OrganicContourSystem {
    pub fn new(binary_map: DynamicImage, hex_layout: HexLayout, config: ContourConfig) -> Self {
        Self {
            binary_map,
            hex_layout,
            config,
            hex_cells: HashMap::new(),
            perlin: Perlin::new(42),
        }
    }

    // Échantillonne la binary map pour un hexagone donné
    fn sample_binary_map(&self, hex: Hex) -> f32 {
        let world_pos = self.hex_layout.hex_to_world_pos(hex);

        // Convertir en coordonnées de la binary map
        let img_x = (world_pos.x * self.config.pixels_per_hex
            + self.binary_map.width() as f32 / 2.0) as u32;
        let img_y = (world_pos.y * self.config.pixels_per_hex
            + self.binary_map.height() as f32 / 2.0) as u32;

        if img_x >= self.binary_map.width() || img_y >= self.binary_map.height() {
            return 0.0;
        }

        // Récupérer la valeur du pixel (assumant grayscale)
        // let pixel_index = (img_y * self.binary_map.width() + img_x) as usize * 4;
        let pixel = self.binary_map.get_pixel(img_x, img_y);
        pixel[0] as f32 / 255.0
    }

    // Détermine le type de terrain en fonction des voisins
    fn determine_terrain_type(&self, hex: Hex, sample_value: f32) -> (TerrainType, bool) {
        // Compter les voisins terre/mer
        let mut land_neighbors = 0;
        let mut water_neighbors = 0;

        for neighbor in hex.all_neighbors() {
            let neighbor_sample = self.sample_binary_map(neighbor);
            if neighbor_sample > self.config.threshold {
                land_neighbors += 1;
            } else {
                water_neighbors += 1;
            }
        }

        let is_land = sample_value > self.config.threshold;
        let is_border = land_neighbors > 0 && water_neighbors > 0;

        let terrain_type = if is_land {
            if is_border {
                TerrainType::Cliff
            } else {
                TerrainType::Land
            }
        } else {
            if is_border {
                TerrainType::Beach
            } else if land_neighbors > 0 {
                TerrainType::ShallowWater
            } else {
                TerrainType::DeepWater
            }
        };

        (terrain_type, is_border)
    }

    // Initialise les cellules hexagonales
    pub fn initialize_hex_grid(&mut self, radius: u32) {
        for hex in Hex::ZERO.range(radius) {
            let sample_value = self.sample_binary_map(hex);
            let (terrain_type, is_border) = self.determine_terrain_type(hex, sample_value);

            self.hex_cells.insert(
                hex,
                CellData {
                    coord: HexCoord::from_hex(hex),
                    terrain_type,
                    is_border,
                    sample_value,
                    distance_to_edge: 0.0, // Sera calculé après
                },
            );
        }

        // Calculer les distances aux bords
        self.calculate_distance_fields();
    }

    // Calcule le champ de distance pour smooth les transitions
    fn calculate_distance_fields(&mut self) {
        // Simplified distance field - en production utiliser jump flooding
        for hex in self.hex_cells.keys().cloned().collect::<Vec<_>>() {
            if let Some(cell) = self.hex_cells.get(&hex) {
                if cell.is_border {
                    // Les cellules frontières ont distance 0
                    self.hex_cells.get_mut(&hex).unwrap().distance_to_edge = 0.0;
                } else {
                    // Trouver la distance au bord le plus proche
                    let mut min_dist = f32::MAX;
                    for other_hex in self.hex_cells.keys() {
                        if self.hex_cells[other_hex].is_border {
                            let dist = hex.unsigned_distance_to(*other_hex) as f32;
                            min_dist = min_dist.min(dist);
                        }
                    }
                    self.hex_cells.get_mut(&hex).unwrap().distance_to_edge = min_dist;
                }
            }
        }
    }

    // Génère les contours organiques en utilisant la grille duale triangulaire
    pub fn generate_organic_contours(&self) -> Vec<ContourPath> {
        let mut contours = Vec::new();
        let mut visited = HashMap::new();

        // Pour chaque cellule frontière, générer un contour
        for (hex, cell) in &self.hex_cells {
            if !cell.is_border {
                continue;
            }

            // Convertir l'hexagone en 6 triangles via la grille duale
            let triangles = self.hex_to_triangular_dual(*hex);

            for triangle in triangles {
                if visited.contains_key(&triangle.id) {
                    continue;
                }

                // Générer le contour pour ce triangle
                if let Some(contour) = self.generate_triangle_contour(&triangle) {
                    contours.push(contour);
                    visited.insert(triangle.id, true);
                }
            }
        }

        // Lisser les contours avec des splines
        contours
            .into_iter()
            .map(|c| self.smooth_contour_with_splines(c))
            .collect()
    }

    // Convertit un hexagone en triangles de la grille duale
    fn hex_to_triangular_dual(&self, hex: Hex) -> Vec<Triangle> {
        let center = self.hex_layout.hex_to_world_pos(hex);

        let vertices = self.hex_layout.hex_corners(hex).to_vec();
        // Utiliser la méthode de hexx pour obtenir les vertex
        // let vertices: Vec<Vec2> = (0..6)
        //     .map(|i| self.hex_layout.hex_corners(i))
        //     .map(|offset| center + offset)
        //     .collect();

        // Créer 6 triangles depuis le centre vers chaque edge
        let mut triangles = Vec::new();
        for i in 0..6 {
            let v1 = vertices[i];
            let v2 = vertices[(i + 1) % 6];

            triangles.push(Triangle {
                id: TriangleId { hex, index: i },
                vertices: [center, v1, v2],
            });
        }

        triangles
    }

    // Génère un contour pour un triangle donné
    fn generate_triangle_contour(&self, triangle: &Triangle) -> Option<ContourPath> {
        // Échantillonner les valeurs aux sommets du triangle
        let values: Vec<f32> = triangle
            .vertices
            .iter()
            .map(|v| {
                // Convertir position monde en coordonnées image
                let img_x = v.x * self.config.pixels_per_hex + self.binary_map.width() as f32 / 2.0;
                let img_y =
                    v.y * self.config.pixels_per_hex + self.binary_map.height() as f32 / 2.0;

                // Ajouter du bruit fractal pour rendre organique
                let noise = self.fractal_noise(v.x, v.y);

                // Échantillonner avec le bruit
                self.sample_at_position(img_x, img_y) + noise * self.config.noise_amplitude
            })
            .collect();

        // Marching triangles simplifié
        self.marching_triangle(&triangle, &values)
    }

    // Bruit fractal pour rendre les contours organiques
    fn fractal_noise(&self, x: f32, y: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.config.noise_frequency;

        for _ in 0..self.config.noise_octaves {
            value += self
                .perlin
                .get([x as f64 * frequency as f64, y as f64 * frequency as f64])
                as f32
                * amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        value
    }

    // Échantillonne la binary map à une position précise avec interpolation bilinéaire
    fn sample_at_position(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor() as u32;
        let x1 = x.ceil() as u32;
        let y0 = y.floor() as u32;
        let y1 = y.ceil() as u32;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        // Interpolation bilinéaire
        let v00 = self.get_pixel_value(x0, y0);
        let v10 = self.get_pixel_value(x1, y0);
        let v01 = self.get_pixel_value(x0, y1);
        let v11 = self.get_pixel_value(x1, y1);

        let v0 = v00 * (1.0 - fx) + v10 * fx;
        let v1 = v01 * (1.0 - fx) + v11 * fx;

        v0 * (1.0 - fy) + v1 * fy
    }

    fn get_pixel_value(&self, x: u32, y: u32) -> f32 {
        if x >= self.binary_map.width() || y >= self.binary_map.height() {
            return 0.0;
        }
        // let pixel_index = (y * self.binary_map.width() + x) as usize * 4;
        let pixel = self.binary_map.get_pixel(x, y);
        pixel[0] as f32 / 255.0
        
    }

    // Algorithme marching triangles
    fn marching_triangle(&self, triangle: &Triangle, values: &[f32]) -> Option<ContourPath> {
        let threshold = self.config.threshold;

        // Classification du triangle (8 cas possibles)
        let case = (if values[0] > threshold { 1 } else { 0 })
            | (if values[1] > threshold { 2 } else { 0 })
            | (if values[2] > threshold { 4 } else { 0 });

        let mut points = Vec::new();

        match case {
            0 | 7 => return None, // Tout dedans ou tout dehors
            1 => {
                // Vertex 0 dedans, interpoler sur edges 0-1 et 0-2
                points.push(self.interpolate_edge(
                    &triangle.vertices[0],
                    &triangle.vertices[1],
                    values[0],
                    values[1],
                    threshold,
                ));
                points.push(self.interpolate_edge(
                    &triangle.vertices[0],
                    &triangle.vertices[2],
                    values[0],
                    values[2],
                    threshold,
                ));
            }
            2 => {
                // Vertex 1 dedans
                points.push(self.interpolate_edge(
                    &triangle.vertices[1],
                    &triangle.vertices[0],
                    values[1],
                    values[0],
                    threshold,
                ));
                points.push(self.interpolate_edge(
                    &triangle.vertices[1],
                    &triangle.vertices[2],
                    values[1],
                    values[2],
                    threshold,
                ));
            }
            3 => {
                // Vertices 0 et 1 dedans
                points.push(self.interpolate_edge(
                    &triangle.vertices[2],
                    &triangle.vertices[0],
                    values[2],
                    values[0],
                    threshold,
                ));
                points.push(self.interpolate_edge(
                    &triangle.vertices[2],
                    &triangle.vertices[1],
                    values[2],
                    values[1],
                    threshold,
                ));
            }
            4 => {
                // Vertex 2 dedans
                points.push(self.interpolate_edge(
                    &triangle.vertices[2],
                    &triangle.vertices[0],
                    values[2],
                    values[0],
                    threshold,
                ));
                points.push(self.interpolate_edge(
                    &triangle.vertices[2],
                    &triangle.vertices[1],
                    values[2],
                    values[1],
                    threshold,
                ));
            }
            5 => {
                // Vertices 0 et 2 dedans
                points.push(self.interpolate_edge(
                    &triangle.vertices[1],
                    &triangle.vertices[0],
                    values[1],
                    values[0],
                    threshold,
                ));
                points.push(self.interpolate_edge(
                    &triangle.vertices[1],
                    &triangle.vertices[2],
                    values[1],
                    values[2],
                    threshold,
                ));
            }
            6 => {
                // Vertices 1 et 2 dedans
                points.push(self.interpolate_edge(
                    &triangle.vertices[0],
                    &triangle.vertices[1],
                    values[0],
                    values[1],
                    threshold,
                ));
                points.push(self.interpolate_edge(
                    &triangle.vertices[0],
                    &triangle.vertices[2],
                    values[0],
                    values[2],
                    threshold,
                ));
            }
            _ => {}
        }

        if points.len() >= 2 {
            Some(ContourPath {
                points,
                is_closed: false,
            })
        } else {
            None
        }
    }

    // Interpolation linéaire entre deux points
    fn interpolate_edge(&self, p1: &Vec2, p2: &Vec2, v1: f32, v2: f32, threshold: f32) -> Vec2 {
        let t = (threshold - v1) / (v2 - v1);
        *p1 + (*p2 - *p1) * t.clamp(0.0, 1.0)
    }

    // Lisse un contour avec des splines de Catmull-Rom
    fn smooth_contour_with_splines(&self, contour: ContourPath) -> ContourPath {
        if contour.points.len() < 4 {
            return contour;
        }

        let mut smoothed_points = Vec::new();
        let segments = 10; // Nombre de subdivisions par segment

        for i in 0..contour.points.len() {
            let p0 = if i == 0 {
                contour.points[contour.points.len() - 1]
            } else {
                contour.points[i - 1]
            };

            let p1 = contour.points[i];
            let p2 = contour.points[(i + 1) % contour.points.len()];
            let p3 = contour.points[(i + 2) % contour.points.len()];

            // Générer les points de la spline
            for j in 0..segments {
                let t = j as f32 / segments as f32;
                let point = self.catmull_rom_point(p0, p1, p2, p3, t);
                smoothed_points.push(point);
            }
        }

        ContourPath {
            points: smoothed_points,
            is_closed: contour.is_closed,
        }
    }

    // Calcul d'un point sur une spline de Catmull-Rom
    fn catmull_rom_point(&self, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
        let tau = self.config.spline_tension;

        let t2 = t * t;
        let t3 = t2 * t;

        let v0 = (p2 - p0) * tau;
        let v1 = (p3 - p1) * tau;

        let a = p1 * 2.0 - p2 * 2.0 + v0 + v1;
        let b = -p1 * 3.0 + p2 * 3.0 - v0 * 2.0 - v1;
        let c = v0;
        let d = p1;

        a * t3 + b * t2 + c * t + d
    }

    // Génère le mesh final pour le rendu
    pub fn generate_mesh(&self, contours: &[ContourPath]) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();

        // Pour chaque contour, créer une bande de polygones
        for contour in contours {
            let base_index = vertices.len() as u16;

            // Créer une bande avec épaisseur
            let thickness = 0.1; // Ajuster selon besoin

            for (i, point) in contour.points.iter().enumerate() {
                let next = contour.points[(i + 1) % contour.points.len()];
                let direction = (next - *point).normalize();
                let perpendicular = Vec2::new(-direction.y, direction.x) * thickness;

                // Ajouter deux vertices pour créer une bande
                vertices.push([point.x - perpendicular.x, point.y - perpendicular.y, 0.0]);
                vertices.push([point.x + perpendicular.x, point.y + perpendicular.y, 0.0]);

                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);

                uvs.push([i as f32 / contour.points.len() as f32, 0.0]);
                uvs.push([i as f32 / contour.points.len() as f32, 1.0]);

                // Créer les triangles
                if i > 0 {
                    let idx = base_index + (i * 2) as u16;
                    indices.push(idx - 2);
                    indices.push(idx - 1);
                    indices.push(idx);

                    indices.push(idx - 1);
                    indices.push(idx + 1);
                    indices.push(idx);
                }
            }

            // Fermer le contour si nécessaire
            if contour.is_closed && contour.points.len() > 2 {
                let last_idx = base_index + ((contour.points.len() - 1) * 2) as u16;
                indices.push(last_idx);
                indices.push(last_idx + 1);
                indices.push(base_index);

                indices.push(last_idx + 1);
                indices.push(base_index + 1);
                indices.push(base_index);
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U16(indices))
    }
}
