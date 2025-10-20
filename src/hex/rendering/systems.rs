use bevy::color::palettes::basic::RED;
use bevy::picking::prelude::*;
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
    // hex_config: Res<HexConfig>,
    // color_tint_materials: Res<ColorTintMaterials>,
    // existing: Query<&HexTile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // let existing_coords: std::collections::HashSet<_> = existing.iter().map(|h| h.coord).collect();

    // for q in -3..3 {
    //     for r in -3..3 {
    //         let hex_coord = HexCoord::new(q, r);
    //         if existing_coords.contains(&hex_coord) {
    //             continue;
    //         }

    //         let hex = hex_coord.to_hex();
    //         let world_pos = hex_config.layout.hex_to_world_pos(hex);
    //         info!("Spawning hex tile {:?} at: {:?}", hex_coord, world_pos);

    //         let visuals = HexVisuals::new(hex_coord);
    //         let material = color_tint_materials.get_material("default".to_string());
    //         let hover_material = color_tint_materials.get_material("hover".to_string());
    //         let pressed_material = color_tint_materials.get_material("pressed".to_string());

    //         let material_clone = material.clone();
    //         let hover_material_clone = hover_material.clone();
    //         let pressed_material_clone = pressed_material.clone();
    //         commands
    //             .spawn((
    //                 Name::new(format!("tile{}_{}", hex_coord.q, hex_coord.r)),
    //                 HexTile { coord: hex_coord },
    //                 visuals.clone(),
    //                 Mesh2d(color_tint_materials.hex_mesh.clone()),
    //                 MeshMaterial2d(material.clone()),
    //                 Transform::from_translation(world_pos.extend(0.0)),
    //                 Pickable::default(),
    //             ))
    //             .observe(|over: On<Pointer<Over>>| {
    //                 println!("Greetings");
    //             })
    //             .observe(click_observer);
    //         // .observe(update_material_on::<Pointer<Over>>(hover_material_clone.clone()))
    //         // .observe(update_material_on::<Pointer<Out>>(material_clone))
    //         // .observe(update_material_on::<Pointer<Press>>(
    //         //     pressed_material_clone,
    //         // ))
    //         // .observe(update_material_on::<Pointer<Release>>(
    //         //     hover_material_clone.clone(),
    //         // ));
    //     }
    // }

    commands
        .spawn((
            Name::new("Circle"),
            Mesh2d(meshes.add(Circle::new(50.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(RED))),
        ))
        .observe(|over: On<Pointer<Over>>| {
            println!("overed");
        });
}

pub fn check_pickable(
    backend_settings: Res<MeshPickingSettings>,
    pickables: Query<&Pickable>,
){
    // info!("Check pickable");
    // info!("Backend settings");
    // info!("requires markers: {}", backend_settings.require_markers);
    // for pickable in pickables {
    //     info!("Is overable: {}", pickable.is_hoverable);
    // }
}

fn click_observer(interaction: On<Pointer<Over>>, query: Query<&Name>) {
    info!("CLICKED!");
    let Ok(name) = query.get(interaction.event_target()) else {
        info!("Failed to get tile");
        return;
    };
    info!("Clicked on: {}", name);
}

fn update_material_on<E: EntityEvent>(
    new_material: Handle<ColorMaterial>,
) -> impl Fn(On<E>, Query<&mut MeshMaterial2d<ColorMaterial>>) {
    move |event, mut query| {
        let Ok(mut mesh_material) = query.get_mut(event.event_target()) else {
            return;
        };
        info!("  Changing material...");
        mesh_material.0 = new_material.clone();
    }
}
