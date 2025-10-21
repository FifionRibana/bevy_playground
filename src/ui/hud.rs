// =============================================================================
// UI - HUD (Heads-Up Display)
// =============================================================================

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct FrameTimeText;

#[derive(Component)]
pub struct EntityCountText;

pub fn setup_hud(mut commands: Commands) {
    // Root node pour HUD
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .with_children(|parent| {
            // Top-left: FPS
            parent
                .spawn((
                    Text::new("FPS: -- (avg: --)"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.0, 0.0)),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.0),
                        left: Val::Px(10.0),
                        ..default()
                    },
                    FpsText,
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: false,
                    },
                ))
                .observe(|over: On<Pointer<Over>>| {
                    println!("oveerd");
                });

            parent.spawn((
                Text::new("Frame time: --ms"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(30.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                FrameTimeText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));

            parent.spawn((
                Text::new("Total entities: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                EntityCountText,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                },
            ));
        });
}

pub fn update_diagnostic_texts(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<(
        &mut Text,
        Option<&FpsText>,
        Option<&FrameTimeText>,
        Option<&EntityCountText>,
    )>,
) {
    let (fps_value, average_fps) =
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                (fps.value().unwrap_or(0.0), value)
            } else {
                (fps.value().unwrap_or(0.0), -1.0)
            }
        } else {
            (-1.0, -1.0)
        };

    let frame_time_value =
        if let Some(frame_time) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
            if let Some(value) = frame_time.smoothed() {
                value * 1000.0
            } else {
                -1.0
            }
        } else {
            -1.0
        };

    let entity_count_value =
        if let Some(entity_count) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT) {
            entity_count.value().unwrap_or(0.0) as usize
        } else {
            0.0 as usize
        };

    for (mut text, fps_query, frame_time_query, entity_count_query) in &mut query {
        if fps_query.is_some() {
            **text = format!("FPS: {:.1} (avg: {:.1})", fps_value, average_fps);
        } else if frame_time_query.is_some() {
            **text = format!("Frame time: {:.2}ms", frame_time_value);
        } else if entity_count_query.is_some() {
            **text = format!("Total entities: {}", entity_count_value);
        }
    }
}
