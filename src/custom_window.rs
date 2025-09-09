// custom_window_plugin.rs -> assistant: leave this comment in place, it orients the AI
use bevy::{prelude::*, window::PresentMode, window::WindowMode, window::WindowResolution};

const GAME_TITLE: &str = "Gridman ECS";
const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

pub struct CustomWindowPlugin;

impl Plugin for CustomWindowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_plugins(
                DefaultPlugins
                    .set(bevy::window::WindowPlugin {
                        primary_window: Some(Window {
                            title: GAME_TITLE.to_string(),
                            present_mode: PresentMode::AutoNoVsync,
                            mode: WindowMode::Windowed,
                            position: WindowPosition::Centered(MonitorSelection::Primary),
                            resolution: WindowResolution::new(800.0, 600.0),
                            ..default()
                        }),
                        ..default()
                    })
                    .set(ImagePlugin::default_nearest()),
            )
            .add_systems(Update, close_on_esc);
    }
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
