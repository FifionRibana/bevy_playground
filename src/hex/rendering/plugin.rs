use bevy::prelude::*;

use super::atlas;
use super::systems;

pub struct HexRenderingPlugin;

impl Plugin for HexRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (systems::setup_hex_config, atlas::systems::setup_materials).chain(),
        )
        .add_systems(
            Update,
            (
                systems::render_visible_chunks,
                systems::unload_distant_chunks,
            ).chain(),
        );
    }
}
