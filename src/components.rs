use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash, States, Debug)]
pub enum GameState {
    #[default]
    Loading,
    Title,
    Playing,
}

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct Velocity {
    pub velocity: Vec2,
}

#[derive(Event)]
pub struct PlayerDied;

#[derive(Resource)]
pub struct GameSpeed {
    pub value: f32,
}

pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSpeed { value: 1.0 }).add_systems(
            Update,
            (update_velocity)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn update_velocity(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.velocity.x * time.delta_secs();
        transform.translation.y += velocity.velocity.y * time.delta_secs();
    }
}
