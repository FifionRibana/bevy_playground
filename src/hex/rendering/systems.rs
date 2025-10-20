use bevy::color::palettes::tailwind::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use hexx::{Hex, HexLayout, HexOrientation};
use std::ops::Deref;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use super::atlas::ColorTintMaterials;
use super::components::{HexTile, HexVisuals};
use super::config::HexConfig;
use crate::camera::MainCamera;
use crate::hex::HexCoord;

pub fn setup_hex_config(mut commands: Commands) {
    let radius = 48.;
    let orientation = HexOrientation::Flat;
    let ratio = Vec2::new(1.0, 0.67);
    let config = HexConfig::new(radius, orientation, ratio);
    commands.insert_resource(config);
    info!(
        "✓ HexConfig configuré (rayon: {}, orientation: {:?}, ratio: {:?})",
        radius, orientation, ratio
    );
}

#[derive(Resource)]
pub struct HexGrid {
    pub entities: HashMap<Hex, Entity>,
    pub layout: HexLayout,
    pub default_mat: Handle<ColorMaterial>,
    pub selected_mat: Handle<ColorMaterial>,
}

pub fn spawn_hex_sprites(
    mut commands: Commands,
    hex_config: Res<HexConfig>,
    color_tint_materials: Res<ColorTintMaterials>,
    existing: Query<&HexTile>,
) {
    let existing_coords: std::collections::HashSet<_> = existing.iter().map(|h| h.coord).collect();

    let material = color_tint_materials.get_material("default".to_string());
    let hover_material = color_tint_materials.get_material("hover".to_string());
    let pressed_material = color_tint_materials.get_material("pressed".to_string());

    for q in -3..3 {
        for r in -3..3 {
            let hex_coord = HexCoord::new(q, r);
            if existing_coords.clone().contains(&hex_coord) {
                continue;
            }

            let hex = hex_coord.to_hex();
            let world_pos = hex_config.layout.hex_to_world_pos(hex);
            info!("Spawning hex tile {:?} at: {:?}", hex_coord, world_pos);

            let visuals = HexVisuals::new(hex_coord);

            // let material_clone = material.clone();
            // let hover_material_clone = hover_material.clone();
            // let pressed_material_clone = pressed_material.clone();
            spawn_hex_sprite(&mut commands, hex_coord, world_pos, color_tint_materials.clone(), existing_coords.clone());
            // commands
            //     .spawn((
            //         Name::new(format!("tile{}_{}", hex_coord.q, hex_coord.r)),
            //         HexTile { coord: hex_coord },
            //         visuals.clone(),
            //         Mesh2d(color_tint_materials.hex_mesh.clone()),
            //         MeshMaterial2d(material.clone()),
            //         Transform::from_translation(world_pos.extend(0.0)),
            //     ))
            //     .observe(update_material_on::<Pointer<Over>>(hover_material.clone()))
            //     .observe(update_material_on::<Pointer<Out>>(material.clone()))
            //     .observe(update_material_on::<Pointer<Press>>(
            //         pressed_material.clone(),
            //     ))
            //     .observe(update_material_on::<Pointer<Release>>(
            //         hover_material.clone(),
            //     ))
            //     .observe(click_handler);
        }
    }
}

pub fn spawn_hex_sprite(commands: &mut Commands,
    hex_coord: HexCoord,
    world_pos: Vec2,
    color_tint_materials: ColorTintMaterials,
    existing_coords: std::collections::HashSet<HexCoord>,
) {
    if existing_coords.contains(&hex_coord) {
        return ;
    }

    let material = color_tint_materials.get_material("default".to_string());
    let hover_material = color_tint_materials.get_material("hover".to_string());
    let pressed_material = color_tint_materials.get_material("pressed".to_string());

    let visuals = HexVisuals::new(hex_coord);

    commands
        .spawn((
            Name::new(format!("tile{}_{}", hex_coord.q, hex_coord.r)),
            HexTile { coord: hex_coord },
            visuals.clone(),
            Mesh2d(color_tint_materials.hex_mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_translation(world_pos.extend(0.0)),
        ))
        .observe(update_material_on::<Pointer<Over>>(hover_material.clone()))
        .observe(update_material_on::<Pointer<Out>>(material.clone()))
        .observe(click_handler)
        .observe(update_material_on::<Pointer<Press>>(
            pressed_material.clone(),
        ))
        .observe(update_material_on::<Pointer<Release>>(
            hover_material.clone(),
        ));
}

fn click_handler(click: On<Pointer<Click>>, mut commands: Commands) {
    if click.button != PointerButton::Secondary {
        return;
    }

    commands.entity(click.event_target()).despawn();
}

fn update_material_on<E: EntityEvent>(
    new_material: Handle<ColorMaterial>,
) -> impl Fn(On<E>, Query<&mut MeshMaterial2d<ColorMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |event, mut query| {
        if let Ok(mut material) = query.get_mut(event.event_target()) {
            material.0 = new_material.clone();
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn spawn_on_click(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
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
            return Ok(())
        }
        spawn_hex_sprite(&mut commands, hex_coord, world_pos, color_tint_materials.clone(), existing_coords.clone());
    }
    Ok(())
}

pub fn handle_input(
    mut commands: Commands<'_, '_>,
    windows: Query<&Window, With<PrimaryWindow>>,
    // cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    grid: Res<HexGrid>,
    mut current_hex: Local<Hex>,
) -> Result {
    let window = windows.single()?;
    let (camera, camera_transform) = cameras.single()?;
    if let Some(position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(camera_transform, p).ok())
    {
        // info!("Position: {}", position);
        let hex_position = grid.layout.world_pos_to_hex(position);
        // info!("Hex position: {:?}", hex_position);
        //     info!("Current hex: {:?}", current_hex);

        if !grid.entities.contains_key(&*current_hex) {
            return Ok(());
        }
        // if hex_position == *current_hex {
        //     return Ok(());
        // }

        commands
            .entity(grid.entities[current_hex.deref()])
            .insert(MeshMaterial2d(grid.default_mat.clone()));

        if !grid.entities.contains_key(&hex_position) {
            return Ok(());
        }

        commands
            .entity(grid.entities[&hex_position.clone()])
            .insert(MeshMaterial2d(grid.selected_mat.clone()));

        *current_hex = hex_position;
    }
    Ok(())
}
