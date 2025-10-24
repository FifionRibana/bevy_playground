use bevy::prelude::*;

use super::super::systems;

pub struct OrganicContourPlugin;

impl Plugin for OrganicContourPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (systems::setup_organic_contour).chain(),
        );
    }
}
