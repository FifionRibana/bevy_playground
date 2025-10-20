use bevy::prelude::*;

use super::handlers;

pub struct HexInputPlugin;

impl Plugin for HexInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handlers::spawn_on_click);
    }
}
