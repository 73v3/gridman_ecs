// src/grid_reservation.rs
use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState};
use crate::tilemap::{MapOffset, TileOffset, HALF_HEIGHT, HALF_WIDTH, TILE_SIZE};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// When set to true, spawns a sprite for each grid cell reservation for debugging.
const VISUAL_DEBUG_RESERVATIONS: bool = true;

pub struct GridReservationPlugin;

impl Plugin for GridReservationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridReservations>()
            // This system runs after all other updates, ensuring that it catches any
            // entities that were despawned during the frame.
            .add_systems(PostUpdate, cleanup_dangling_reservations);

        // If the debug flag is enabled, add the visualization systems.
        if VISUAL_DEBUG_RESERVATIONS {
            app.add_systems(
                Update,
                (sync_reservation_visuals, update_visualizer_positions)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
        }
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

/// A marker component for the visual sprite representing a reservation.
/// Stores the grid position it corresponds to.
#[derive(Component)]
struct ReservationVisualizer(IVec2);

/// Spawns and despawns sprites to match the current state of GridReservations.
fn sync_reservation_visuals(
    mut commands: Commands,
    reservations: Res<GridReservations>,
    game_assets: Res<GameAssets>,
    // Query for all existing visualizer entities
    visualizer_query: Query<(Entity, &ReservationVisualizer)>,
) {
    // Collect all grid positions that are currently reserved.
    let needed_visuals: HashSet<IVec2> = reservations.0.keys().cloned().collect();

    // Collect all grid positions that currently have a visualizer sprite.
    let mut current_visuals: HashMap<IVec2, Entity> = HashMap::new();
    for (entity, visualizer) in &visualizer_query {
        current_visuals.insert(visualizer.0, entity);
    }

    // Despawn unneeded visualizers by finding which current ones are no longer needed.
    for (pos, entity) in &current_visuals {
        if !needed_visuals.contains(pos) {
            // Use .despawn() which is idiomatic for Bevy 0.16+
            commands.entity(*entity).despawn();
        }
    }

    // Spawn new visualizers where needed by finding which needed ones don't exist yet.
    for pos in needed_visuals {
        if !current_visuals.contains_key(&pos) {
            commands.spawn((
                Sprite {
                    image: game_assets.reservation_texture.clone(),
                    ..default()
                },
                ReservationVisualizer(pos),
                // GameEntity ensures it's cleaned up when we exit the Playing state.
                GameEntity,
                // The transform will be set correctly by the update_visualizer_positions system.
                // A high Z-value ensures it renders on top of the floor and player.
                Transform::from_xyz(0.0, 0.0, 1.5),
            ));
        }
    }
}

/// Updates the world-space transform of each visualizer sprite based on its grid position
/// and the current camera scroll offsets.
fn update_visualizer_positions(
    map_offset: Res<MapOffset>,
    tile_offset: Res<TileOffset>,
    mut query: Query<(&ReservationVisualizer, &mut Transform)>,
) {
    for (visualizer, mut trans) in &mut query {
        let pos = visualizer.0;

        // This calculation is identical to how other grid-based entities are positioned,
        // ensuring the debug sprite is perfectly centered on the tile.
        let x = (pos.x as f32 - map_offset.0.x as f32 - HALF_WIDTH) * TILE_SIZE + tile_offset.0.x;
        let y = (pos.y as f32 - map_offset.0.y as f32 - HALF_HEIGHT) * TILE_SIZE + tile_offset.0.y;

        trans.translation.x = x;
        trans.translation.y = y;
    }
}

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
