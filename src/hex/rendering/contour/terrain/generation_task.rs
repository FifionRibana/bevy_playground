use bevy::prelude::*;
use bevy::tasks::Task;

use super::mesh_data::TerrainMeshData;

#[derive(Resource)]
pub struct TerrainGenerationTask(Task<TerrainMeshData>);
