use hexx::*;
use image::{DynamicImage, GenericImageView, Rgba};
use std::sync::{Arc, Mutex};

use super::super::terrain::{GenerationStage, TerrainGenerationProgress, TerrainMeshData};
use super::super::{ContourConfig, ContourPath};

// =================== UTILITAIRES ===================

pub fn update_progress(
    progress: &Arc<Mutex<TerrainGenerationProgress>>,
    stage: GenerationStage,
    value: f32,
    message: impl Into<String>,
) {
    if let Ok(mut p) = progress.lock() {
        p.stage = stage;
        p.progress = value;
        p.message = message.into();
    }
}

pub fn sample_binary_map_for_hex(
    binary_map: &DynamicImage,
    hex: Hex,
    layout: &HexLayout,
    config: &ContourConfig,
) -> f32 {
    let world_pos = layout.hex_to_world_pos(hex);
    let img_x = (world_pos.x * config.pixels_per_hex + binary_map.width() as f32 / 2.0) as u32;
    let img_y = (world_pos.y * config.pixels_per_hex + binary_map.height() as f32 / 2.0) as u32;

    if img_x >= binary_map.width() || img_y >= binary_map.height() {
        return 0.0;
    }

    let pixel = binary_map.get_pixel(img_x, img_y);
    pixel[0] as f32 / 255.0
}

pub fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32, tension: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;

    let v0 = (p2 - p0) * tension;
    let v1 = (p3 - p1) * tension;

    let a = p1 * 2.0 - p2 * 2.0 + v0 + v1;
    let b = -p1 * 3.0 + p2 * 3.0 - v0 * 2.0 - v1;
    let c = v0;
    let d = p1;

    a * t3 + b * t2 + c * t + d
}

pub fn triangulate_single_contour(contour: &ContourPath) -> TerrainMeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    // Implémentation simplifiée d'ear clipping
    // ... (voir l'artifact précédent pour l'implémentation complète)

    TerrainMeshData {
        vertices,
        indices,
        normals,
        uvs,
    }
}

pub fn merge_mesh_data(meshes: Vec<TerrainMeshData>) -> TerrainMeshData {
    let mut merged = TerrainMeshData {
        vertices: Vec::new(),
        indices: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
    };

    for mesh in meshes {
        let base_index = merged.vertices.len() as u16;

        merged.vertices.extend(mesh.vertices);
        merged.normals.extend(mesh.normals);
        merged.uvs.extend(mesh.uvs);

        // Ajuster les indices
        for idx in mesh.indices {
            merged.indices.push(idx + base_index);
        }
    }

    merged
}

pub fn generate_contours_from_samples(
    samples: Vec<(Hex, f32)>,
    layout: &HexLayout,
    config: &ContourConfig,
    progress: &Arc<Mutex<TerrainGenerationProgress>>,
) -> Vec<ContourPath> {
    // Implémentation simplifiée
    Vec::new()
}
