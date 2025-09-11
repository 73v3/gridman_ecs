use crate::collider::ProjectileCollision;
use crate::components::{EnemyDied, GameState, PlayerDied};
use crate::enemy::Enemy;
use crate::player::Player;
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
    mut player_died_events: EventWriter<PlayerDied>,
    mut enemy_died_events: EventWriter<EnemyDied>,
    // Query to determine if the victim was a Player or an Enemy.
    victim_query: Query<(Has<Player>, Has<Enemy>, &Transform)>,
) {
    for event in collision_events.read() {
        // Despawn the projectile on any confirmed collision.
        commands.entity(event.projectile).despawn();

        // Check what the victim was and react accordingly.
        if let Ok((is_player, is_enemy, transform)) = victim_query.get(event.victim) {
            let pos = transform.translation;
            if is_player {
                commands.entity(event.victim).despawn();
                player_died_events.write(PlayerDied(pos));
                info!("Player was hit by a projectile!");
            } else if is_enemy {
                commands.entity(event.victim).despawn();
                enemy_died_events.write(EnemyDied(pos));
            }
        }
    }
}
