use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::VisibilityClass;
use bevy::prelude::*;
use bevy::window::PresentMode;
// mod hex::rendering;
use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};
use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod camera;
mod hex;
mod shared;
mod state;
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
                })
                .set(AssetPlugin {
                    file_path: "assets".to_string(),
                    ..default()
                }),
            // .set(bevy::log::LogPlugin {
            //     filter: "bevy_dev_tools=trace".into(),
            //     ..default()
            // }),
            MeshPickingPlugin,
            DebugPickingPlugin,
        ))
        .register_type::<Aabb>()
        .register_type::<GlobalTransform>()
        .register_type::<InheritedVisibility>()
        .register_type::<Mesh2d>()
        .register_type::<Name>()
        .register_type::<Transform>()
        .register_type::<ViewVisibility>()
        .register_type::<Visibility>()
        .register_type::<VisibilityClass>()
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins((
            camera::CameraPlugin,
            // state::StatePlugin,
            // hex::rendering::HexRenderingPlugin,
            hex::rendering::contour::OrganicContourPlugin,
            // hex::input::HexInputPlugin,
            // ui::UiPlugin,
        ))
        .add_plugins((
            // LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
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
