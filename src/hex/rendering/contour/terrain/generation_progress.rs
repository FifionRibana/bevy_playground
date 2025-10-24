use bevy::prelude::*;
use std::sync::{Arc, Mutex};

use super::GenerationStage;

#[derive(Resource)]
pub struct TerrainGenerationProgress {
    pub stage: GenerationStage,
    pub progress: f32,
    pub message: String,
}

