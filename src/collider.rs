// collider.rs
use crate::components::{EnemyDied, GameState, PlayerDied};
use crate::enemy::Enemy;
use crate::grid_movement::GridMover;
use crate::grid_reservation::GridReservations;
use crate::player::Player;
use crate::projectile::{Bouncable, Projectile};
use bevy::prelude::*;

/// Component representing a collider with a size for AABB collision detection.
#[derive(Component)]
pub struct Collider {
    pub size: Vec2,
}

/// Event triggered when a projectile collides with another entity.
#[derive(Event)]
pub struct ProjectileCollision {
    pub projectile: Entity,
    pub victim: Entity,
}

/// The eight adjacent directions (cardinal and diagonal) for adjacency checks.
const DIRECTIONS: [IVec2; 8] = [
    IVec2::new(0, 1),   // Up
    IVec2::new(0, -1),  // Down
    IVec2::new(-1, 0),  // Left
    IVec2::new(1, 0),   // Right
    IVec2::new(-1, 1),  // Up-Left
    IVec2::new(1, 1),   // Up-Right
    IVec2::new(-1, -1), // Down-Left
    IVec2::new(1, -1),  // Down-Right
];

/// Expansion factor for player and enemy colliders during AABB checks.
const COLLIDER_EXPANSION_FACTOR: f32 = 2.25;

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileCollision>().add_systems(
            Update,
            (
                check_projectile_collisions,
                check_player_enemy_adjacency
                    .after(crate::grid_movement::MovementSystems::UpdateMover),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Checks for collisions between projectiles and other entities using the grid reservation system.
/// This is a highly efficient, targeted collision detection method.
fn check_projectile_collisions(
    mut events: EventWriter<ProjectileCollision>,
    reservations: Res<GridReservations>,
    projectiles: Query<(Entity, &Transform, &Collider, &GridMover, &Bouncable), With<Projectile>>,
    collidables: Query<(&Transform, &Collider)>,
    player_query: Query<(), With<Player>>,
) {
    for (proj_entity, proj_transform, proj_collider, proj_mover, bouncable) in &projectiles {
        // A projectile is only a threat if it's actively moving towards a new tile.
        if proj_mover.direction == IVec2::ZERO {
            continue;
        }

        // --- Broad Phase ---
        // Determine the tile the projectile is moving into.
        let target_tile = proj_mover.grid_pos + proj_mover.direction;

        // Check if this target tile is reserved by another entity.
        if let Some(&victim_entity) = reservations.0.get(&target_tile) {
            // --- Narrow Phase ---
            // We have a potential collision. Get the victim's components.
            // The .get() method on a Query is highly optimized.
            if let Ok((victim_transform, victim_collider)) = collidables.get(victim_entity) {
                // Check if the victim is the player and if the projectile hasn't bounced at least once.
                let is_player = player_query.get(victim_entity).is_ok();
                let bounced = bouncable.initial.saturating_sub(bouncable.remaining);
                if is_player && bounced < 1 {
                    continue; // Skip collision with player if projectile hasn't bounced.
                }

                // Perform the precise AABB check.
                if aabb_overlap(
                    proj_transform.translation.xy(),
                    proj_collider.size,
                    victim_transform.translation.xy(),
                    victim_collider.size,
                ) {
                    // Collision confirmed. Write the event.
                    events.write(ProjectileCollision {
                        projectile: proj_entity,
                        victim: victim_entity,
                    });
                }
            }
        }
    }
}

/// Checks for AABB overlap between the player and enemies in adjacent grid cells with expanded collider sizes.
/// Triggers player and enemy death if an overlap is detected.
fn check_player_enemy_adjacency(
    mut commands: Commands,
    mut player_died_events: EventWriter<PlayerDied>,
    mut enemy_died_events: EventWriter<EnemyDied>,
    player_query: Query<(Entity, &GridMover, &Transform, &Collider), With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Collider), With<Enemy>>,
    reservations: Res<GridReservations>,
) {
    if let Ok((player_entity, player_mover, player_transform, player_collider)) =
        player_query.single()
    {
        // Check each adjacent cell using the constant DIRECTIONS array.
        for &dir in DIRECTIONS.iter() {
            let adjacent_pos = player_mover.grid_pos + dir;
            if let Some(&enemy_entity) = reservations.0.get(&adjacent_pos) {
                // Confirm the entity is an enemy.
                if let Ok((enemy_entity, enemy_transform, enemy_collider)) =
                    enemy_query.get(enemy_entity)
                {
                    // Perform AABB overlap check with expanded collider sizes.
                    if aabb_overlap(
                        player_transform.translation.xy(),
                        player_collider.size * COLLIDER_EXPANSION_FACTOR,
                        enemy_transform.translation.xy(),
                        enemy_collider.size * COLLIDER_EXPANSION_FACTOR,
                    ) {
                        // Collision detected; despawn both and trigger death events.
                        commands.entity(player_entity).despawn();
                        commands.entity(enemy_entity).despawn();
                        player_died_events.write(PlayerDied(player_transform.translation));
                        enemy_died_events.write(EnemyDied(enemy_transform.translation));
                        info!(
                            "Player died due to AABB overlap with enemy at {:?}",
                            adjacent_pos
                        );
                        // Break after first collision to avoid multiple death events in one frame.
                        break;
                    }
                }
            }
        }
    }
}

/// Checks for overlap between two Axis-Aligned Bounding Boxes.
pub fn aabb_overlap(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let half1 = size1 / 2.0;
    let half2 = size2 / 2.0;
    let min1 = pos1 - half1;
    let max1 = pos1 + half1;
    let min2 = pos2 - half2;
    let max2 = pos2 + half2;

    min1.x < max2.x && max1.x > min2.x && min1.y < max2.y && max1.y > min2.y
}
