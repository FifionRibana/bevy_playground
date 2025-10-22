use bevy::prelude::*;

use super::main_camera;
use super::controller;
use super::CameraSettings;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraSettings>()
            .add_systems(Startup, main_camera::setup_camera)
            .add_systems(Update, (controller::camera_movement, controller::camera_zoom));
    }
}