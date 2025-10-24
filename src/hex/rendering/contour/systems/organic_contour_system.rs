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
        HexConfig, HexCoord,
        rendering::contour::{ContourConfig, ContourPath},
    },
    shared::types::{CellData, TerrainType, Triangle, TriangleId},
};

pub fn setup_organic_contour(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    hex_config: Res<HexConfig>,
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
        pixels_per_hex: 0.25, // 4 hex par pixel
        noise_amplitude: 0.2, // Force du bruit
        noise_frequency: 3.0, // Fréquence du bruit
        noise_octaves: 8,     // Détail fractal
        threshold: 0.5,       // Seuil terre/mer
        spline_tension: 0.5,  // Courbure des splines
    };

    // Créer le système
    let layout = hex_config.layout.clone();
    let mut system = OrganicContourSystem::new(binary_map, layout, config);

    // Initialiser la grille
    system.initialize_hex_grid(100);

    // Générer les contours
    let contours = system.generate_organic_contours_global();
    // let contours = system.generate_organic_contours();
    info!("Generated {} contours", contours.len());

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

    // Méthode alternative : générer des contours globaux avec marching squares
    pub fn generate_organic_contours_global(&self) -> Vec<ContourPath> {
        // Créer une grille régulière pour l'échantillonnage
        let grid_size = 2000; // Résolution de la grille d'échantillonnage
        let bounds = self.calculate_bounds();
        let cell_size = Vec2::new(
            (bounds.1.x - bounds.0.x) / grid_size as f32,
            (bounds.1.y - bounds.0.y) / grid_size as f32,
        );

        // Échantillonner la binary map + bruit sur une grille régulière
        let mut grid_values = vec![vec![0.0; grid_size + 1]; grid_size + 1];

        for y in 0..=grid_size {
            for x in 0..=grid_size {
                let world_pos = Vec2::new(
                    bounds.0.x + x as f32 * cell_size.x,
                    bounds.0.y + y as f32 * cell_size.y,
                );

                // Échantillonner la binary map avec bruit fractal
                let img_x = (world_pos.x * self.config.pixels_per_hex
                    + self.binary_map.width() as f32 / 2.0);
                let img_y = (world_pos.y * self.config.pixels_per_hex
                    + self.binary_map.height() as f32 / 2.0);

                let base_value = self.sample_at_position(img_x, img_y);
                let noise =
                    self.fractal_noise(world_pos.x, world_pos.y) * self.config.noise_amplitude;

                grid_values[y][x] = base_value + noise;
            }
        }

        // Appliquer marching squares
        let mut contours = Vec::new();

        for y in 0..grid_size {
            for x in 0..grid_size {
                let corners = [
                    grid_values[y][x],
                    grid_values[y][x + 1],
                    grid_values[y + 1][x + 1],
                    grid_values[y + 1][x],
                ];

                if let Some(segments) =
                    self.marching_square_cell(x, y, &corners, cell_size, bounds.0)
                {
                    for segment in segments {
                        contours.push(ContourPath {
                            points: vec![segment.0, segment.1],
                            is_closed: false,
                        });
                    }
                }
            }
        }

        // Connecter et lisser les contours
        let connected = self.connect_segments_to_contours(
            contours
                .into_iter()
                .flat_map(|c| {
                    c.points
                        .windows(2)
                        .map(|w| (w[0], w[1]))
                        .collect::<Vec<_>>()
                })
                .collect(),
        );

        connected
            .into_iter()
            .map(|c| self.smooth_contour_with_splines(c))
            .collect()
    }

    // Génère les contours organiques en utilisant la grille duale triangulaire
    pub fn generate_organic_contours(&self) -> Vec<ContourPath> {
        let mut segments = Vec::new();
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
                    segments.extend(contour.points.windows(2).map(|w| (w[0], w[1])));
                    visited.insert(triangle.id, true);
                }
            }
        }

        // Connecter les segments pour former des contours fermés
        let contours = self.connect_segments_to_contours(segments);

        // Lisser les contours avec des splines
        contours
            .into_iter()
            .map(|c| self.smooth_contour_with_splines(c))
            .collect()
    }

    // Calcule les limites de la carte
    fn calculate_bounds(&self) -> (Vec2, Vec2) {
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);

        for hex in self.hex_cells.keys() {
            let pos = self.hex_layout.hex_to_world_pos(*hex);
            let size = self.hex_layout.scale;
            min = min.min(pos - size);
            max = max.max(pos + size);
        }

        (min, max)
    }

    // Marching squares pour une cellule
    fn marching_square_cell(
        &self,
        x: usize,
        y: usize,
        corners: &[f32; 4],
        cell_size: Vec2,
        offset: Vec2,
    ) -> Option<Vec<(Vec2, Vec2)>> {
        let threshold = self.config.threshold;

        // Classification de la cellule
        let case = (if corners[0] > threshold { 1 } else { 0 })
            | (if corners[1] > threshold { 2 } else { 0 })
            | (if corners[2] > threshold { 4 } else { 0 })
            | (if corners[3] > threshold { 8 } else { 0 });

        let base = offset + Vec2::new(x as f32 * cell_size.x, y as f32 * cell_size.y);

        let mut segments = Vec::new();

        // Positions des points interpolés sur les edges
        let interp = |v1: f32, v2: f32| (threshold - v1) / (v2 - v1);

        match case {
            0 | 15 => return None,
            1 | 14 => {
                let a = base + Vec2::new(0.0, cell_size.y * interp(corners[0], corners[3]));
                let b = base + Vec2::new(cell_size.x * interp(corners[0], corners[1]), 0.0);
                segments.push((a, b));
            }
            2 | 13 => {
                let a = base + Vec2::new(cell_size.x * interp(corners[0], corners[1]), 0.0);
                let b = base + Vec2::new(cell_size.x, cell_size.y * interp(corners[1], corners[2]));
                segments.push((a, b));
            }
            3 | 12 => {
                let a = base + Vec2::new(0.0, cell_size.y * interp(corners[0], corners[3]));
                let b = base + Vec2::new(cell_size.x, cell_size.y * interp(corners[1], corners[2]));
                segments.push((a, b));
            }
            4 | 11 => {
                let a = base + Vec2::new(cell_size.x, cell_size.y * interp(corners[1], corners[2]));
                let b = base + Vec2::new(cell_size.x * interp(corners[3], corners[2]), cell_size.y);
                segments.push((a, b));
            }
            5 => {
                // Cas ambigü - choisir une diagonale
                let a = base + Vec2::new(0.0, cell_size.y * interp(corners[0], corners[3]));
                let b = base + Vec2::new(cell_size.x * interp(corners[0], corners[1]), 0.0);
                segments.push((a, b));
                let c = base + Vec2::new(cell_size.x, cell_size.y * interp(corners[1], corners[2]));
                let d = base + Vec2::new(cell_size.x * interp(corners[3], corners[2]), cell_size.y);
                segments.push((c, d));
            }
            6 | 9 => {
                let a = base + Vec2::new(cell_size.x * interp(corners[0], corners[1]), 0.0);
                let b = base + Vec2::new(cell_size.x * interp(corners[3], corners[2]), cell_size.y);
                segments.push((a, b));
            }
            7 | 8 => {
                let a = base + Vec2::new(0.0, cell_size.y * interp(corners[0], corners[3]));
                let b = base + Vec2::new(cell_size.x * interp(corners[3], corners[2]), cell_size.y);
                segments.push((a, b));
            }
            10 => {
                // Autre cas ambigü
                let a = base + Vec2::new(cell_size.x * interp(corners[0], corners[1]), 0.0);
                let b = base + Vec2::new(cell_size.x, cell_size.y * interp(corners[1], corners[2]));
                segments.push((a, b));
                let c = base + Vec2::new(0.0, cell_size.y * interp(corners[0], corners[3]));
                let d = base + Vec2::new(cell_size.x * interp(corners[3], corners[2]), cell_size.y);
                segments.push((c, d));
            }
            _ => {}
        }

        if segments.is_empty() {
            None
        } else {
            Some(segments)
        }
    }

    // Connecte les segments individuels en contours fermés
    fn connect_segments_to_contours(&self, segments: Vec<(Vec2, Vec2)>) -> Vec<ContourPath> {
        let mut contours = Vec::new();
        let mut used = vec![false; segments.len()];
        let epsilon = 0.001; // Tolérance pour connecter les points

        for start_idx in 0..segments.len() {
            if used[start_idx] {
                continue;
            }

            let mut path = vec![segments[start_idx].0, segments[start_idx].1];
            used[start_idx] = true;

            // Essayer de connecter d'autres segments
            loop {
                let last_point = *path.last().unwrap();
                let mut found = false;

                for (idx, segment) in segments.iter().enumerate() {
                    if used[idx] {
                        continue;
                    }

                    // Vérifier si ce segment se connecte
                    if (segment.0 - last_point).length() < epsilon {
                        path.push(segment.1);
                        used[idx] = true;
                        found = true;
                        break;
                    } else if (segment.1 - last_point).length() < epsilon {
                        path.push(segment.0);
                        used[idx] = true;
                        found = true;
                        break;
                    }
                }

                if !found {
                    break;
                }

                // Vérifier si on a fermé le contour
                if path.len() > 3 && (path[0] - last_point).length() < epsilon {
                    contours.push(ContourPath {
                        points: path[..path.len() - 1].to_vec(), // Enlever le dernier point dupliqué
                        is_closed: true,
                    });
                    break;
                }
            }

            // Si le contour n'est pas fermé mais a une longueur suffisante
            if path.len() > 2 && !contours.last().map_or(false, |c| c.is_closed) {
                contours.push(ContourPath {
                    points: path,
                    is_closed: false,
                });
            }
        }

        contours
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

        // Générer un mesh plein pour toutes les cellules de terre
        // Trianguler chaque contour fermé (îles de terre)
        for contour in contours {
            if contour.points.len() < 3 {
                continue;
            }

            // Utiliser ear clipping pour trianguler le polygone
            self.triangulate_polygon(
                &contour.points,
                &mut vertices,
                &mut indices,
                &mut normals,
                &mut uvs,
            );
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

    // Triangulation par ear clipping pour les polygones organiques
    fn triangulate_polygon(
        &self,
        points: &[Vec2],
        vertices: &mut Vec<[f32; 3]>,
        indices: &mut Vec<u16>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
    ) {
        if points.len() < 3 {
            return;
        }

        let base_index = vertices.len() as u16;

        // Calculer le centre et les bounds pour les UVs
        let mut center = Vec2::ZERO;
        let mut min = points[0];
        let mut max = points[0];

        for point in points {
            center += *point;
            min = min.min(*point);
            max = max.max(*point);
        }
        center /= points.len() as f32;
        let size = max - min;

        // Ajouter tous les vertices du contour
        for point in points {
            vertices.push([point.x, point.y, 0.0]);
            normals.push([0.0, 0.0, 1.0]);

            // UVs basées sur la position relative
            let uv = (*point - min) / size;
            uvs.push([uv.x, uv.y]);
        }

        // Ear clipping simplifié
        let mut remaining: Vec<usize> = (0..points.len()).collect();

        while remaining.len() > 3 {
            let mut ear_found = false;

            for i in 0..remaining.len() {
                let prev = remaining[(i + remaining.len() - 1) % remaining.len()];
                let curr = remaining[i];
                let next = remaining[(i + 1) % remaining.len()];

                if self.is_ear(&points, prev, curr, next, &remaining) {
                    // Créer le triangle
                    indices.push(base_index + prev as u16);
                    indices.push(base_index + curr as u16);
                    indices.push(base_index + next as u16);

                    // Retirer le sommet du milieu
                    remaining.remove(i);
                    ear_found = true;
                    break;
                }
            }

            // Sécurité: si aucune oreille n'est trouvée, forcer la triangulation
            if !ear_found && remaining.len() > 3 {
                indices.push(base_index + remaining[0] as u16);
                indices.push(base_index + remaining[1] as u16);
                indices.push(base_index + remaining[2] as u16);
                remaining.remove(1);
            }
        }

        // Ajouter le dernier triangle
        if remaining.len() == 3 {
            indices.push(base_index + remaining[0] as u16);
            indices.push(base_index + remaining[1] as u16);
            indices.push(base_index + remaining[2] as u16);
        }
    }

    // Vérifie si un triangle forme une "oreille" valide
    fn is_ear(
        &self,
        points: &[Vec2],
        prev: usize,
        curr: usize,
        next: usize,
        remaining: &[usize],
    ) -> bool {
        let p1 = points[prev];
        let p2 = points[curr];
        let p3 = points[next];

        // Vérifier que le triangle est orienté correctement (CCW)
        let area = (p2.x - p1.x) * (p3.y - p1.y) - (p3.x - p1.x) * (p2.y - p1.y);
        if area <= 0.0 {
            return false;
        }

        // Vérifier qu'aucun autre point n'est à l'intérieur du triangle
        for &idx in remaining {
            if idx == prev || idx == curr || idx == next {
                continue;
            }

            if self.point_in_triangle(points[idx], p1, p2, p3) {
                return false;
            }
        }

        true
    }

    // Test si un point est à l'intérieur d'un triangle
    fn point_in_triangle(&self, p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
        let v0 = c - a;
        let v1 = b - a;
        let v2 = p - a;

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        (u >= 0.0) && (v >= 0.0) && (u + v <= 1.0)
    }

    // Ajoute une bordure visible pour les contours (optionnel)
    fn add_contour_overlay(
        &self,
        vertices: &mut Vec<[f32; 3]>,
        indices: &mut Vec<u16>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        contours: &[ContourPath],
    ) {
        let elevation = 0.01; // Légèrement au-dessus pour éviter z-fighting
        let thickness = 0.05; // Épaisseur de la bordure

        for contour in contours {
            if contour.points.len() < 2 {
                continue;
            }

            let base_index = vertices.len() as u16;

            for (i, point) in contour.points.iter().enumerate() {
                let next = if i < contour.points.len() - 1 {
                    contour.points[i + 1]
                } else if contour.is_closed {
                    contour.points[0]
                } else {
                    continue;
                };

                let direction = (next - *point).normalize();
                let perpendicular = Vec2::new(-direction.y, direction.x) * thickness;

                // Quatre vertices pour un segment de ligne épais
                vertices.push([
                    point.x - perpendicular.x,
                    point.y - perpendicular.y,
                    elevation,
                ]);
                vertices.push([
                    point.x + perpendicular.x,
                    point.y + perpendicular.y,
                    elevation,
                ]);

                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);

                let t = i as f32 / contour.points.len() as f32;
                uvs.push([t, 0.0]);
                uvs.push([t, 1.0]);

                // Créer les triangles si on a au moins 4 vertices
                if i > 0 {
                    let idx = base_index + (i * 2) as u16;
                    // Premier triangle
                    indices.push(idx - 2);
                    indices.push(idx - 1);
                    indices.push(idx);
                    // Second triangle
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
    }
}
