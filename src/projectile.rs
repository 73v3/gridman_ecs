use crate::collider::ProjectileCollision;
use crate::components::{GameState, PlayerDied};
use crate::enemy::Enemy;
use crate::player::Player;
use crate::score::ScoreChanged;
use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct Bouncable {
    pub initial: u32,   // Tracks the initial number of bounces allowed
    pub remaining: u32, // Tracks the remaining bounces
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_projectile_collisions.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Listens for `ProjectileCollision` events and handles the consequences.
fn handle_projectile_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileCollision>,
    mut score_events: EventWriter<ScoreChanged>,
    mut player_died_events: EventWriter<PlayerDied>,
    // Query to determine if the victim was a Player or an Enemy.
    victim_query: Query<(Has<Player>, Has<Enemy>)>,
) {
    for event in collision_events.read() {
        // Despawn the projectile on any confirmed collision.
        // commands.get_entity() returns a Result, so we use `if let Ok`.
        if let Ok(mut entity_commands) = commands.get_entity(event.projectile) {
            entity_commands.despawn();
        }

        // Check what the victim was and react accordingly.
        if let Ok((is_player, is_enemy)) = victim_query.get(event.victim) {
            if is_player {
                if let Ok(mut entity_commands) = commands.get_entity(event.victim) {
                    entity_commands.despawn();
                }
                // Use .write() to send events, as .send() is deprecated.
                player_died_events.write(PlayerDied);
                info!("Player was hit by a projectile!");
            } else if is_enemy {
                if let Ok(mut entity_commands) = commands.get_entity(event.victim) {
                    entity_commands.despawn();
                }
                // Use .write() to send events.
                score_events.write(ScoreChanged);
            }
        }
    }
}
