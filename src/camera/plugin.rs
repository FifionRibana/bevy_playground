use bevy::prelude::*;

use super::main_camera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, main_camera::setup_camera);
    }
}