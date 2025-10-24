use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct TerrainSettings {
    pub low_res_scale: f32,      // 0.25 = 1/4 de la résolution
    pub upscale_smoothing: bool, // Lisser lors de l'agrandissement
    pub parallel_threshold: usize, // Nombre min d'éléments pour paralléliser
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            low_res_scale: 0.25,
            upscale_smoothing: true,
            parallel_threshold: 100,
        }
    }
}