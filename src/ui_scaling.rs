use crate::resolution::Resolution;
use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::{PrimaryWindow, WindowResized};

pub struct UiScalingPlugin;

impl Plugin for UiScalingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui_scale)
            .add_systems(Update, update_ui_scale_on_resize);
    }
}

fn setup_ui_scale(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    resolution: Res<Resolution>,
) {
    let initial_scale = if let Ok(window) = window_query.single() {
        window.resolution.height() / resolution.base_resolution.y
    } else {
        1.0
    };
    commands.insert_resource(UiScale(initial_scale));
}

fn update_ui_scale_on_resize(
    mut resize_events: EventReader<WindowResized>,
    mut ui_scale: ResMut<UiScale>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    resolution: Res<Resolution>,
) {
    for _event in resize_events.read() {
        if let Ok(primary_window) = window_query.single() {
            let current_height = primary_window.resolution.height();
            ui_scale.0 = current_height / resolution.base_resolution.y;
        }
    }
}
