use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use hexx::*;
use image::{DynamicImage, GenericImageView, Rgba};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use super::super::terrain::{
    GenerationStage, TerrainGenerationProgress, TerrainMeshData, TerrainSettings,
};
use super::super::{ContourConfig, ContourPath};
use super::utilities;

use crate::hex::HexConfig;

#[derive(Resource)]
struct TerrainGenerationTask(Task<TerrainMeshData>);

#[derive(Resource)]
struct GenerationProgressHandle(Arc<Mutex<TerrainGenerationProgress>>);

pub fn start_terrain_generation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    thread_pool: Res<AsyncComputeTaskPool>,
    hex_config: Res<HexConfig>,
    terrain_settings: Res<TerrainSettings>,
) {
    // Initialiser le progress
    commands.insert_resource(TerrainGenerationProgress {
        stage: GenerationStage::LoadingImage,
        progress: 0.0,
        message: "Chargement de l'image...".to_string(),
    });

    // Charger l'image
    let image_path = "assets/maps/Gaulyia_binarymap.png";
    let binary_map = image::open(image_path).expect("Failed to load binary map image");

    // Configuration
    let config = ContourConfig {
        pixels_per_hex: 0.25 * terrain_settings.low_res_scale,
        noise_amplitude: 0.3,
        noise_frequency: 2.0,
        noise_octaves: 3,
        threshold: 0.5,
        spline_tension: 0.5,
    };

    let layout = hex_config.layout.clone();
    let hex_radius = 100;

    // Créer la tâche asynchrone
    let progress = Arc::new(Mutex::new(TerrainGenerationProgress {
        stage: GenerationStage::LoadingImage,
        progress: 0.0,
        message: "Initialisation...".to_string(),
    }));

    let progress_clone = progress.clone();
    let settings_clone = terrain_settings.clone();

    let task = thread_pool.spawn(async move {
        generate_terrain_async(
            binary_map,
            config,
            layout,
            hex_radius,
            progress_clone,
            settings_clone,
        )
        .await
    });

    commands.insert_resource(TerrainGenerationTask(task));
    commands.insert_resource(GenerationProgressHandle(progress));
}

// =================== GÉNÉRATION ASYNCHRONE ===================

async fn generate_terrain_async(
    binary_map: DynamicImage,
    config: ContourConfig,
    layout: HexLayout,
    hex_radius: u32,
    progress: Arc<Mutex<TerrainGenerationProgress>>,
    settings: TerrainSettings,
) -> TerrainMeshData {
    // Simuler l'attente du chargement de l'image
    // En production, il faudrait vraiment attendre que l'asset soit chargé
    utilities::update_progress(
        &progress,
        GenerationStage::LoadingImage,
        1.0,
        "Image chargée",
    );

    // Phase 1: Génération basse résolution
    utilities::update_progress(
        &progress,
        GenerationStage::SamplingTerrain,
        0.0,
        "Échantillonnage du terrain...",
    );

    let low_res_data = generate_low_resolution(
        binary_map.clone(),
        config.clone(),
        layout,
        hex_radius,
        &progress,
        &settings,
    );

    // Phase 2: Upscaling et amélioration
    utilities::update_progress(
        &progress,
        GenerationStage::GeneratingContours,
        0.0,
        "Génération des contours...",
    );

    let high_res_data = upscale_terrain_data(
        low_res_data,
        settings.low_res_scale,
        settings.upscale_smoothing,
        &progress,
    );

    // Phase 3: Triangulation finale
    utilities::update_progress(
        &progress,
        GenerationStage::TriangulatingMesh,
        0.0,
        "Triangulation du mesh...",
    );

    let mesh_data = triangulate_mesh(high_res_data, &progress, &settings);

    utilities::update_progress(
        &progress,
        GenerationStage::Complete,
        1.0,
        "Génération terminée!",
    );

    mesh_data
}

// =================== GÉNÉRATION BASSE RÉSOLUTION ===================

fn generate_low_resolution(
    binary_map: DynamicImage,
    mut config: ContourConfig,
    layout: HexLayout,
    hex_radius: u32,
    progress: &Arc<Mutex<TerrainGenerationProgress>>,
    settings: &TerrainSettings,
) -> Vec<ContourPath> {
    // Réduire la résolution
    config.pixels_per_hex *= settings.low_res_scale;
    let reduced_radius = (hex_radius as f32 * settings.low_res_scale) as u32;

    // Générer les hexagones en parallèle
    let hex_coords: Vec<Hex> = Hex::ZERO.range(reduced_radius).collect();
    let total = hex_coords.len();

    // Paralléliser l'échantillonnage si assez d'éléments
    let hex_cells = if total > settings.parallel_threshold {
        hex_coords
            .par_iter()
            .enumerate()
            .map(|(i, &hex)| {
                if i % 100 == 0 {
                    let prog = i as f32 / total as f32;
                    utilities::update_progress(
                        progress,
                        GenerationStage::SamplingTerrain,
                        prog,
                        format!("Échantillonnage: {}/{}", i, total),
                    );
                }

                let sample_value =
                    utilities::sample_binary_map_for_hex(&binary_map, hex, &layout, &config);
                (hex, sample_value)
            })
            .collect::<Vec<_>>()
    } else {
        hex_coords
            .iter()
            .map(|&hex| {
                let sample_value =
                    utilities::sample_binary_map_for_hex(&binary_map, hex, &layout, &config);
                (hex, sample_value)
            })
            .collect()
    };

    // Générer les contours (cette partie est difficile à paralléliser)
    utilities::generate_contours_from_samples(hex_cells, &layout, &config, progress)
}

// =================== UPSCALING ===================

fn upscale_terrain_data(
    low_res_contours: Vec<ContourPath>,
    scale_factor: f32,
    smooth: bool,
    progress: &Arc<Mutex<TerrainGenerationProgress>>,
) -> Vec<ContourPath> {
    let inv_scale = 1.0 / scale_factor;
    let total = low_res_contours.len();

    // Paralléliser l'upscaling des contours
    let upscaled: Vec<ContourPath> = low_res_contours
        .par_iter()
        .enumerate()
        .map(|(i, contour)| {
            if i % 10 == 0 {
                utilities::update_progress(
                    progress,
                    GenerationStage::GeneratingContours,
                    i as f32 / total as f32,
                    format!("Upscaling contour {}/{}", i, total),
                );
            }

            let mut new_points = Vec::new();

            for window in contour.points.windows(2) {
                let p1 = window[0] * inv_scale;
                let p2 = window[1] * inv_scale;

                if smooth {
                    // Subdiviser et lisser
                    let subdivisions = 4;
                    for j in 0..subdivisions {
                        let t = j as f32 / subdivisions as f32;
                        let interpolated = p1 + (p2 - p1) * t;

                        // Ajouter une petite perturbation pour plus de naturel
                        let noise = Vec2::new(
                            (i * 1000 + j * 10) as f32 * 0.001 % 1.0 - 0.5,
                            (i * 1000 + j * 10 + 1) as f32 * 0.001 % 1.0 - 0.5,
                        ) * 0.1;

                        new_points.push(interpolated + noise);
                    }
                } else {
                    new_points.push(p1);
                }
            }

            // Ajouter le dernier point
            if let Some(&last) = contour.points.last() {
                new_points.push(last * inv_scale);
            }

            ContourPath {
                points: new_points,
                is_closed: contour.is_closed,
            }
        })
        .collect();

    // Appliquer un lissage supplémentaire si demandé
    if smooth {
        smooth_contours_parallel(upscaled, progress)
    } else {
        upscaled
    }
}

// =================== LISSAGE PARALLÈLE ===================

fn smooth_contours_parallel(
    contours: Vec<ContourPath>,
    progress: &Arc<Mutex<TerrainGenerationProgress>>,
) -> Vec<ContourPath> {
    let total = contours.len();

    contours
        .par_iter()
        .enumerate()
        .map(|(i, contour)| {
            if i % 10 == 0 {
                utilities::update_progress(
                    progress,
                    GenerationStage::SmoothingContours,
                    i as f32 / total as f32,
                    format!("Lissage {}/{}", i, total),
                );
            }

            apply_catmull_rom_smoothing(contour)
        })
        .collect()
}

fn apply_catmull_rom_smoothing(contour: &ContourPath) -> ContourPath {
    if contour.points.len() < 4 {
        return contour.clone();
    }

    let mut smoothed = Vec::new();
    let segments = 5;

    for i in 0..contour.points.len() {
        let p0 = contour.points[(i + contour.points.len() - 1) % contour.points.len()];
        let p1 = contour.points[i];
        let p2 = contour.points[(i + 1) % contour.points.len()];
        let p3 = contour.points[(i + 2) % contour.points.len()];

        for j in 0..segments {
            let t = j as f32 / segments as f32;
            smoothed.push(utilities::catmull_rom(p0, p1, p2, p3, t, 0.5));
        }
    }

    ContourPath {
        points: smoothed,
        is_closed: contour.is_closed,
    }
}

// =================== TRIANGULATION PARALLÈLE ===================

fn triangulate_mesh(
    contours: Vec<ContourPath>,
    progress: &Arc<Mutex<TerrainGenerationProgress>>,
    settings: &TerrainSettings,
) -> TerrainMeshData {
    let total = contours.len();

    // Triangulation parallèle par contour
    let triangulated: Vec<TerrainMeshData> = if contours.len() > settings.parallel_threshold {
        contours
            .par_iter()
            .enumerate()
            .filter_map(|(i, contour)| {
                if i % 10 == 0 {
                    utilities::update_progress(
                        progress,
                        GenerationStage::TriangulatingMesh,
                        i as f32 / total as f32,
                        format!("Triangulation {}/{}", i, total),
                    );
                }

                if contour.is_closed && contour.points.len() >= 3 {
                    Some(utilities::triangulate_single_contour(contour))
                } else {
                    None
                }
            })
            .collect()
    } else {
        contours
            .iter()
            .filter_map(|contour| {
                if contour.is_closed && contour.points.len() >= 3 {
                    Some(utilities::triangulate_single_contour(contour))
                } else {
                    None
                }
            })
            .collect()
    };

    // Fusionner tous les meshes
    utilities::merge_mesh_data(triangulated)
}
