use bevy::prelude::*;

use super::atlas;
use super::systems;

pub struct HexRenderingPlugin;

impl Plugin for HexRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                systems::setup_hex_config,
                atlas::systems::setup_materials,
                systems::spawn_hex_sprites,
            )
                .chain(),
        );
    }
}
