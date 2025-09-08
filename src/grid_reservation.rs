// src/grid_reservation.rs
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct GridReservationPlugin;

impl Plugin for GridReservationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridReservations>()
            // This system runs after all other updates, ensuring that it catches any
            // entities that were despawned during the frame.
            .add_systems(PostUpdate, cleanup_dangling_reservations);
    }
}

/// A resource that stores a map of reserved grid cells to the entity reserving them.
/// This provides a fast, centralized lookup for collision avoidance.
#[derive(Resource, Default)]
pub struct GridReservations(pub HashMap<IVec2, Entity>);

/// A marker component for entities that should reserve their grid cells.
/// Entities with this component will be unable to move into cells reserved
/// by other entities that also have this component.
#[derive(Component)]
pub struct GridReserver;

/// A system that cleans up reservations for entities that have been despawned
/// or have had their `GridReserver` component removed.
///
/// This prevents "ghost" reservations from permanently blocking tiles.
fn cleanup_dangling_reservations(
    mut reservations: ResMut<GridReservations>,
    mut removed_reservers: RemovedComponents<GridReserver>,
) {
    // Collect the removed entities into a HashSet for efficient O(1) lookups.
    // In Bevy 0.16, you must use the .read() method to get an iterator.
    let removed_set: HashSet<Entity> = removed_reservers.read().collect();

    // No need to run if no components were removed this frame.
    if removed_set.is_empty() {
        return;
    }

    // Create a temporary Vec of cells to clear. We do this to avoid borrowing `reservations`
    // mutably while iterating over it.
    let cells_to_clear: Vec<IVec2> = reservations
        .0
        .iter()
        // Find all reservations where the entity ID is in our set of removed entities.
        .filter(|(_, &entity)| removed_set.contains(&entity))
        .map(|(&cell, _)| cell)
        .collect();

    for cell in cells_to_clear {
        reservations.0.remove(&cell);
    }
}
