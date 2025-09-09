use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use std::time::Duration;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_fps_display)
            .add_systems(
                Update,
                (update_fps_display, test_clear).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct FpsText;

fn setup_fps_display(mut commands: Commands, game_assets: Res<GameAssets>) {
    info!("Setting up FPS display");
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font: game_assets.font.clone(),
            font_size: 8.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)), // Light gray for minimalist look
        TextLayout::new_with_justify(JustifyText::Left),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        FpsText,
        GameEntity, // Ensures cleanup when exiting GameState::Playing
    ));
}

fn update_fps_display(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    time: Res<Time>,
    mut timer: Local<Timer>, // Local timer to track update interval
) {
    // Update every 0.5 seconds
    timer.tick(Duration::from_secs_f32(time.delta_secs()));
    if !timer.just_finished() {
        return;
    }
    timer.set_duration(Duration::from_secs_f32(0.5));
    timer.reset();

    if let Ok(mut text) = query.single_mut() {
        // Get FPS from diagnostics
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.0 = format!("FPS: {:.0}", fps);
        } else {
            info!("FPS diagnostic not available");
            text.0 = "FPS: --".to_string();
        }
    }
}

fn test_clear(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::End) {
        info!("END pressed");
    }
}
