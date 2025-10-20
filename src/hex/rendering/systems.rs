use bevy::prelude::*;
use hexx::HexOrientation;

use super::atlas::ColorTintMaterials;
use super::components::{HexTile, HexVisuals};
use super::config::HexConfig;
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

pub fn spawn_hex_sprites(
    mut commands: Commands,
    hex_config: Res<HexConfig>,
    color_tint_materials: Res<ColorTintMaterials>,
    existing: Query<&HexTile>,
) {
    let existing_coords: std::collections::HashSet<_> = existing.iter().map(|h| h.coord).collect();

    for q in -3..3 {
        for r in -3..3 {
            let hex_coord = HexCoord::new(q, r);
            if existing_coords.clone().contains(&hex_coord) {
                continue;
            }

            let hex = hex_coord.to_hex();
            let world_pos = hex_config.layout.hex_to_world_pos(hex);
            // info!("Spawning hex tile {:?} at: {:?}", hex_coord, world_pos);

            spawn_hex_sprite(
                &mut commands,
                hex_coord,
                world_pos,
                color_tint_materials.clone(),
                existing_coords.clone(),
            );
        }
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
