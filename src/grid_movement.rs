// grid_movement.rs

//! This module defines the core logic for entity movement on a grid.
//!
//! It introduces the `GridMover` component, which tracks an entity's discrete grid position
//! and its progress toward the next tile. The module provides systems to update this state
//! based on an `IntendedDirection` (set by player input or AI), handle wall collisions,
//! and translate the logical grid position into a smooth, visual world position (`Transform`).
//! A `SystemSet` is used to ensure a deterministic order of operations for movement logic.

use bevy::ecs::schedule::SystemSet;
use bevy::prelude::*;

use crate::components::GameState;
use crate::grid_reservation::{GridReservations, GridReserver};
use crate::map::MapData;
use crate::projectile::{Bouncable, Projectile};
use crate::tilemap::{MapOffset, TileOffset, HALF_HEIGHT, HALF_WIDTH, TILE_SIZE};

/// A component that enables grid-based movement for an entity.
#[derive(Component)]
pub struct GridMover {
    /// The entity's current position in integer grid coordinates.
    pub grid_pos: IVec2,
    /// The direction the entity is currently moving (e.g., (1, 0) for right).
    /// A zero vector indicates the entity is stationary.
    pub direction: IVec2,
    /// The progress (0.0 to 1.0) of the movement from `grid_pos` to the next tile.
    /// 0.0 means the entity is perfectly on `grid_pos`; 1.0 means it has arrived at the next tile.
    pub progress: f32,
    /// The speed of the entity, measured in how many pixels it would travel per second.
    /// This is used to calculate the increment of `progress` each frame.
    pub speed: f32,
}

/// A component representing the desired direction of movement for an entity.
///
/// This is decoupled from `GridMover.direction` to allow for input buffering.
/// For example, a player can press a new direction key before the entity has
/// finished moving to the current tile.
#[derive(Component)]
pub struct IntendedDirection(pub IVec2);

/// Defines a strict order of execution for systems related to movement.
///
/// This is crucial to prevent issues like one-frame delays between input and movement,
/// or the camera position being updated before the player's transform. The `.chain()`
/// ensures these sets run sequentially within a single frame.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MovementSystems {
    /// Handles reading input from players or AI to set `IntendedDirection`.
    Input,
    /// Updates the `GridMover` state (progress, direction) based on `IntendedDirection`.
    UpdateMover,
    /// Translates the `GridMover` state into a world-space `Transform` for rendering.
    UpdatePosition,
    /// Adjusts camera/viewport scrolling based on the final entity position.
    AdjustScroll,
    /// Applies any changes to offsets to entity positions.
    ApplyOffsetChanges,
}

/// The plugin that adds all grid movement logic to the application.
pub struct GridMovementPlugin;

impl Plugin for GridMovementPlugin {
    fn build(&self, app: &mut App) {
        app
            // Configure the order of our system sets.
            .configure_sets(
                Update,
                (
                    MovementSystems::Input,
                    MovementSystems::UpdateMover.after(MovementSystems::Input),
                    MovementSystems::UpdatePosition.after(MovementSystems::UpdateMover),
                    MovementSystems::AdjustScroll.after(MovementSystems::UpdatePosition),
                    MovementSystems::ApplyOffsetChanges.after(MovementSystems::AdjustScroll),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            // Add the systems to their respective sets.
            .add_systems(
                Update,
                update_grid_movement.in_set(MovementSystems::UpdateMover),
            )
            .add_systems(
                Update,
                update_grid_positions.in_set(MovementSystems::UpdatePosition),
            )
            .add_systems(
                Update,
                update_grid_positions
                    .run_if(resource_changed::<MapOffset>.or(resource_changed::<TileOffset>))
                    .in_set(MovementSystems::ApplyOffsetChanges),
            );
    }
}

/// The core system that updates the state of all `GridMover` components.
///
/// This system functions like a state machine for each moving entity. It handles:
/// - Starting movement from a standstill.
/// - Advancing movement progress frame-by-frame.
/// - Reaching a destination tile and deciding what to do next (stop, continue, or change direction).
/// - Handling collisions with walls, including logic for bouncing projectiles.
#[allow(clippy::too_many_arguments)] // Bevy systems often require many parameters.
fn update_grid_movement(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut GridMover,
        &mut IntendedDirection,
        Option<&GridReserver>,
        Option<&mut Bouncable>,
        Option<&Projectile>,
    )>,
    time: Res<Time>,
    map_data: Res<MapData>,
    mut reservations: ResMut<GridReservations>,
) {
    for (entity, mut mover, mut intended, reserver, bouncable, projectile) in &mut query {
        // --- State 1: Entity is stationary ---
        if mover.direction == IVec2::ZERO {
            let new_dir = intended.0;
            if new_dir != IVec2::ZERO {
                let next_tile = mover.grid_pos + new_dir;

                // Check if the target tile is valid for movement.
                let is_tile_wall = is_wall(next_tile, &map_data);
                let mut is_tile_reserved = false;

                // Only check for reservations if the entity is a GridReserver.
                if reserver.is_some() {
                    if let Some(&occupant) = reservations.0.get(&next_tile) {
                        // A tile is only considered reserved if it's occupied by another entity.
                        is_tile_reserved = occupant != entity;
                    }
                }

                // Only start moving if the target tile is not a wall and not reserved.
                if !is_tile_wall && !is_tile_reserved {
                    mover.direction = new_dir;
                    mover.progress = 0.0;
                    // If this is a reserver, claim the destination tile.
                    if reserver.is_some() {
                        reservations.0.insert(next_tile, entity);
                    }
                }
            }
        // --- State 2: Entity is currently moving between tiles ---
        } else {
            // Calculate how much to increment progress this frame.
            // Diagonal movement is faster, so we normalize by the vector length.
            let dir_vec = mover.direction.as_vec2();
            let dist_factor = dir_vec.length();
            if dist_factor == 0.0 {
                continue; // Avoid division by zero if direction is somehow zero here.
            }
            let inc = mover.speed * time.delta_secs() / (TILE_SIZE * dist_factor);
            mover.progress += inc;

            // --- State 3: Entity has arrived at or passed the destination tile ---
            if mover.progress >= 1.0 {
                let old_pos = mover.grid_pos;
                let current_direction = mover.direction;
                mover.grid_pos += current_direction; // Lock position to the new grid tile.

                // If this entity reserves tiles, free the one it just left.
                if reserver.is_some() {
                    // Only remove the reservation if this entity was the one holding it.
                    if let Some(&occupant) = reservations.0.get(&old_pos) {
                        if occupant == entity {
                            reservations.0.remove(&old_pos);
                        }
                    }
                }

                // Check if the entity wants to continue in the same direction.
                let is_continuing =
                    intended.0 == current_direction && current_direction != IVec2::ZERO;

                if is_continuing {
                    let next_tile = mover.grid_pos + current_direction;
                    if !is_wall(next_tile, &map_data) {
                        // Path is clear: carry over the "excess" progress for a smooth transition.
                        mover.progress -= 1.0;
                    } else {
                        // Wall detected ahead.
                        let can_bounce = bouncable.as_ref().map_or(false, |b| b.remaining > 0);
                        if can_bounce {
                            // --- Bouncing Logic ---
                            let new_dir =
                                calculate_reflection(current_direction, mover.grid_pos, &map_data);
                            mover.direction = new_dir;
                            intended.0 = new_dir;
                            if let Some(mut b) = bouncable {
                                b.remaining -= 1;
                            }
                            // Adjust progress based on new direction's length to maintain speed.
                            let old_length = current_direction.as_vec2().length();
                            let new_length = new_dir.as_vec2().length();
                            mover.progress -= 1.0;
                            if new_length > 0.0 && old_length > 0.0 {
                                mover.progress *= old_length / new_length;
                            }
                        } else {
                            // Cannot bounce: stop movement.
                            mover.progress = 0.0;
                            mover.direction = IVec2::ZERO;
                            intended.0 = IVec2::ZERO;
                            // If it's a projectile, despawn it on impact.
                            if projectile.is_some() {
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                } else {
                    // Not continuing straight: reset progress and check for a new direction.
                    mover.progress = 0.0;
                    let new_dir = intended.0;
                    if new_dir != IVec2::ZERO {
                        let next_tile = mover.grid_pos + new_dir;
                        if !is_wall(next_tile, &map_data) {
                            mover.direction = new_dir; // Start moving in the new intended direction.
                        } else {
                            mover.direction = IVec2::ZERO; // New direction is blocked, so stop.
                        }
                    } else {
                        mover.direction = IVec2::ZERO; // No new direction, so stop.
                    }
                }
            }
        }
    }
}

/// Calculates a simple reflection vector for bouncing.
///
/// It checks for open paths horizontally and vertically from the point of impact.
/// - If the horizontal path is clear, it reflects vertically (y -> -y).
/// - If the vertical path is clear, it reflects horizontally (x -> -x).
/// - If both are blocked (a corner), it reflects both (x -> -x, y -> -y).
fn calculate_reflection(dir: IVec2, grid_pos: IVec2, map_data: &MapData) -> IVec2 {
    let dx = dir.x;
    let dy = dir.y;

    // Check adjacent tiles in the direction of velocity components.
    let horiz_next = grid_pos + IVec2::new(dx, 0);
    let vert_next = grid_pos + IVec2::new(0, dy);
    let horiz_clear = !is_wall(horiz_next, map_data);
    let vert_clear = !is_wall(vert_next, map_data);

    if horiz_clear {
        IVec2::new(dx, -dy) // Reflect vertically
    } else if vert_clear {
        IVec2::new(-dx, dy) // Reflect horizontally
    } else {
        IVec2::new(-dx, -dy) // Reflect fully (corner hit)
    }
}

/// Translates the logical `GridMover` position into a final `Transform` for rendering.
///
/// This system runs after `update_grid_movement`, ensuring it uses the most up-to-date
/// grid position and progress. It accounts for the global map and tile offsets to correctly
/// position the entity within the camera's viewport.
fn update_grid_positions(
    map_offset: Res<MapOffset>,
    tile_offset: Res<TileOffset>,
    mut query: Query<(&GridMover, &mut Transform)>,
) {
    for (mover, mut trans) in &mut query {
        // Calculate the effective position, including the fractional progress towards the next tile.
        let effective_pos = mover.grid_pos.as_vec2() + mover.direction.as_vec2() * mover.progress;

        // Convert the effective grid position to world coordinates.
        let x =
            (effective_pos.x - map_offset.0.x as f32 - HALF_WIDTH) * TILE_SIZE + tile_offset.0.x;
        let y =
            (effective_pos.y - map_offset.0.y as f32 - HALF_HEIGHT) * TILE_SIZE + tile_offset.0.y;

        trans.translation.x = x;
        trans.translation.y = y;
    }
}

/// A utility function to check if a given grid position is a wall or out of bounds.
///
/// It performs bounds checking and then looks up the tile type in the `MapData` resource.
/// The Y-coordinate is flipped because the map image data is loaded with (0,0) at the top-left,
/// while our grid coordinates treat (0,0) as the bottom-left.
pub fn is_wall(pos: IVec2, map: &MapData) -> bool {
    // Treat any position outside the map boundaries as a wall.
    if pos.x < 0 || pos.y < 0 || pos.x >= map.width as i32 || pos.y >= map.height as i32 {
        return true;
    }
    let x = pos.x as u32;
    let y = pos.y as u32;

    // Flip Y for lookup in the map data vector.
    let flipped_y = map.height - 1 - y;
    let idx = (flipped_y * map.width + x) as usize;

    // Safely get the value, defaulting to `true` (wall) if the index is somehow out of bounds.
    map.is_wall.get(idx).copied().unwrap_or(true)
}
