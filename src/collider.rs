// collider.rs
use crate::components::GameState;
use crate::grid_movement::GridMover;
use crate::grid_reservation::GridReservations;
use crate::player::Player;
use crate::projectile::{Bouncable, Projectile};
use bevy::prelude::*;

#[derive(Component)]
pub struct Collider {
    pub size: Vec2,
}

#[derive(Event)]
pub struct ProjectileCollision {
    pub projectile: Entity,
    pub victim: Entity,
}

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileCollision>().add_systems(
            Update,
            check_projectile_collisions.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Checks for collisions by leveraging the grid reservation system.
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
