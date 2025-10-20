use bevy::prelude::*;
use bevy::window::PresentMode;
// mod hex::rendering;
use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};

use bevy::color::palettes::tailwind::*;

mod camera;
mod hex;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Playground".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
                // .set(bevy::log::LogPlugin {
                //     filter: "bevy_dev_tools=trace".into(),
                //     ..default()
                // }),
            MeshPickingPlugin,
            DebugPickingPlugin,
        ))
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins((
            // camera::CameraPlugin,
            hex::rendering::HexRenderingPlugin,
            // ui::UiPlugin,
        ))
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
        // .add_systems(Startup, setup_scene)
        // .add_systems(
        //     PreUpdate,
        //     (|mut mode: ResMut<DebugPickingMode>| {
        //         *mode = match *mode {
        //             DebugPickingMode::Disabled => DebugPickingMode::Normal,
        //             DebugPickingMode::Normal => DebugPickingMode::Noisy,
        //             DebugPickingMode::Noisy => DebugPickingMode::Disabled,
        //         }
        //     })
        //     .distributive_run_if(bevy::input::common_conditions::input_just_pressed(KeyCode::F3))
        // )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Set up the materials.
    let white_matl = materials.add(Color::WHITE);
    let ground_matl = materials.add(Color::from(GRAY_300));
    let hover_matl = materials.add(Color::from(CYAN_300));
    let pressed_matl = materials.add(Color::from(YELLOW_300));

    commands
        .spawn((
            Name::new("Circle"),
            Mesh2d(meshes.add(Circle::new(50.0))),
            MeshMaterial2d(white_matl.clone()),
        ))
        .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
        .observe(update_material_on::<Pointer<Out>>(white_matl.clone()))
        .observe(update_material_on::<Pointer<Press>>(pressed_matl.clone()))
        .observe(update_material_on::<Pointer<Release>>(hover_matl.clone()));

    // Camera
    // commands.spawn((Camera2d::default(),));
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_material_on<E: EntityEvent>(
    new_material: Handle<ColorMaterial>,
) -> impl Fn(On<E>, Query<&mut MeshMaterial2d<ColorMaterial>>) {
    println!("Hey!");
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |event, mut query| {
        if let Ok(mut material) = query.get_mut(event.event_target()) {
            material.0 = new_material.clone();
        }
    }
}