use bevy::prelude::*;
use super::super::systems;
use super::super::terrain::TerrainSettings;

pub struct OptimizedTerrainPlugin;

impl Plugin for OptimizedTerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(TerrainSettings::default())
            .add_systems(Startup, systems::start_terrain_generation)
            .add_systems(Update, (
                systems::process_terrain_generation,
                // display_progress_ui,
            ));
    }
}