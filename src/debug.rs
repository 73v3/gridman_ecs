use crate::components::GameState;
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, test_clear.run_if(in_state(GameState::Playing)));
    }
}

fn test_clear(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::End) {
        info!("END pressed");
    }
}
