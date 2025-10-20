use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::hex;
use crate::hex::HexCoord;
use crate::hex::rendering::atlas::ColorTintMaterials;
use crate::hex::rendering::components::{HexTile, HexVisuals};
use crate::hex::rendering::config::HexConfig;

pub fn spawn_on_click(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    hex_config: Res<HexConfig>,
    color_tint_materials: Res<ColorTintMaterials>,
    existing: Query<&HexTile>,
) -> Result {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return Ok(());
    }

    let existing_coords: std::collections::HashSet<_> = existing.iter().map(|h| h.coord).collect();

    let window = windows.single()?;
    let (camera, camera_transform) = cameras.single()?;
    if let Some(position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
    {
        // info!("Position: {}", position);
        let hex_position = hex_config.layout.world_pos_to_hex(position);
        let hex_coord = HexCoord::from_hex(hex_position);
        let world_pos = hex_config.layout.hex_to_world_pos(hex_position);

        if existing_coords.contains(&hex_coord) {
            return Ok(());
        }
        hex::rendering::spawn_hex_sprite(
            &mut commands,
            hex_coord,
            world_pos,
            color_tint_materials.clone(),
            existing_coords.clone(),
        );
    }
    Ok(())
}
