// =============================================================================
// State - Plugin
// =============================================================================

use bevy::prelude::*;

use super::cache::WorldCache;
use super::components::StreamingConfig;

use super::streaming;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<WorldCache>()
            .init_resource::<StreamingConfig>()
            .add_systems(Update, (streaming::request_chunks, streaming::unload_distant_chunks));
    }
}
