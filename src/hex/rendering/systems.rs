use bevy::prelude::*;
use hexx::{HexOrientation, shapes};

use super::atlas::ColorTintMaterials;
use super::components::{HexTile, HexVisuals};
use super::config::HexConfig;
use crate::camera::MainCamera;
use crate::hex::HexCoord;

pub fn setup_hex_config(mut commands: Commands) {
    let radius = 48.;
    let orientation = HexOrientation::Flat;
    let ratio = Vec2::new(1.0, 0.67);
    let chunk_size = 3 as u8;
    let config = HexConfig::new(radius, orientation, ratio, chunk_size);
    commands.insert_resource(config);
    info!(
        "✓ HexConfig configuré (rayon: {}, orientation: {:?}, ratio: {:?})",
        radius, orientation, ratio
    );
}

pub fn spawn_hex_sprites(
    mut commands: Commands,
    hex_config: Res<HexConfig>,
    color_tint_materials: Res<ColorTintMaterials>,
    existing: Query<&HexTile>,
) {
    let existing_coords: std::collections::HashSet<_> = existing.iter().map(|h| h.coord).collect();

    spawn_hex_chunk(
        &mut commands,
        hex_config.clone(),
        HexCoord::new(0, 0),
        color_tint_materials.clone(),
        existing_coords.clone(),
        hex_config.chunk_size,
    );

    let hex_coord = HexCoord::new(0, 0);
    let hex = hex_coord.to_hex();
    let world_pos = hex_config.layout.hex_to_world_pos(hex);
    info!("Hex Coord: {:?}", hex_coord);
    info!("Hex      : {:?}", hex);
    info!("World pos: {:?}", world_pos);
    // for q in -3..3 {
    //     for r in -3..3 {
    //         let hex_coord = HexCoord::new(q, r);
    //         if existing_coords.clone().contains(&hex_coord) {
    //             continue;
    //         }

    //         let hex = hex_coord.to_hex();
    //         let world_pos = hex_config.layout.hex_to_world_pos(hex);
    //         // info!("Spawning hex tile {:?} at: {:?}", hex_coord, world_pos);

    //         spawn_hex_sprite(
    //             &mut commands,
    //             hex_coord,
    //             world_pos,
    //             color_tint_materials.clone(),
    //             existing_coords.clone(),
    //         );
    //     }
    // }
}

pub fn spawn_hex_chunk(
    commands: &mut Commands,
    hex_config: HexConfig,
    hex_coord: HexCoord,
    color_tint_materials: ColorTintMaterials,
    existing_coords: std::collections::HashSet<HexCoord>,
    chunk_size: u8,
) {
    let chunk_hex = hex_coord.to_hex().to_lower_res(chunk_size as u32);
    let center_hex = chunk_hex.to_higher_res(chunk_size as u32);

    info!("Chunk : {:?}", HexCoord::from_hex(chunk_hex));
    info!("Spawning chunk at {:?}", HexCoord::from_hex(center_hex));

    let hex_shape = shapes::Hexagon {
        center: center_hex,
        radius: chunk_size as u32,
    };

    for coord in hex_shape.coords() {
        spawn_hex_sprite(
            commands,
            HexCoord::from_hex(coord),
            hex_config.layout.hex_to_world_pos(coord),
            color_tint_materials.clone(),
            existing_coords.clone(),
        )
    }
}

pub fn spawn_hex_sprite(
    commands: &mut Commands,
    hex_coord: HexCoord,
    world_pos: Vec2,
    color_tint_materials: ColorTintMaterials,
    existing_coords: std::collections::HashSet<HexCoord>,
) {
    if existing_coords.contains(&hex_coord) {
        return;
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

fn click_handler(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    hex_config: Res<HexConfig>,
    existing: Query<(Entity, &HexTile)>,
) {
    if click.button == PointerButton::Primary {
        let click_position = click.pointer_location.position;
        info!("Clicked at: {}", click_position);
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, click_position) {
            let clicked_hex = hex_config.layout.world_pos_to_hex(world_pos);
            let clicked_hex_coord = HexCoord::from_hex(clicked_hex);
            let lower_res_hex = clicked_hex.to_lower_res(hex_config.chunk_size as u32);
            let lower_res_hex_coord = HexCoord::from_hex(lower_res_hex);

            let center_hex = lower_res_hex.to_higher_res(hex_config.chunk_size as u32);
            let center_hex_coord = HexCoord::from_hex(center_hex);

            info!("Clicked hex: {:?}", clicked_hex_coord);
            info!("   > lower res: {:?}", lower_res_hex_coord);
            info!("   > center   : {:?}", center_hex_coord);
        }
    } else if click.button == PointerButton::Secondary {
        // commands.entity(click.event_target()).despawn();
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        let click_position = click.pointer_location.position;
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, click_position) {
            let clicked_hex = hex_config.layout.world_pos_to_hex(world_pos);
            let chunk_hex = clicked_hex.to_lower_res(hex_config.chunk_size as u32);
            let center_hex = chunk_hex.to_higher_res(hex_config.chunk_size as u32);

            info!("Despawn chunk: {:?}", HexCoord::from_hex(chunk_hex));

            let hex_shape = shapes::Hexagon {
                center: center_hex,
                radius: hex_config.chunk_size as u32,
            };

            let shape_coords: std::collections::HashSet<_> = hex_shape.coords().collect();

            let entities: Vec<Entity> = existing
                .iter()
                .filter(|(_, hex_tile)| shape_coords.contains(&hex_tile.coord.to_hex()))
                .map(|(entity, _)| entity)
                .collect();

            for entity in entities {
                commands.entity(entity).despawn();
            }
        }
    }
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
