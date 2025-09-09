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

#[derive(Component)]
pub struct Speed {
    pub value: f32,
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
        // Register the PlayerDied event here.
        app.add_event::<PlayerDied>()
            .insert_resource(GameSpeed { value: 1.0 })
            .add_systems(
                Update,
                (update_velocity)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn update_velocity(
    mut query: Query<(&Velocity, &mut Transform, Option<&Speed>)>,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
) {
    for (velocity, mut transform, speed) in query.iter_mut() {
        let speed_modifier = speed.map_or(1.0, |s| s.value) * game_speed.value;
        transform.translation.x += velocity.velocity.x * time.delta_secs() * speed_modifier;
        transform.translation.y += velocity.velocity.y * time.delta_secs() * speed_modifier;
    }
}
