// player.rs

//! Manages the player entity, including its creation, input handling, actions,
//! and the camera scrolling logic that follows it.

use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::audio;
use crate::collider::Collider;
use crate::components::{GameEntity, GameState};
use crate::grid_movement::{is_wall, GridMover, IntendedDirection, MovementSystems};
use crate::grid_reservation::{GridReservations, GridReserver};
use crate::map::MapData;
use crate::projectile::{Bouncable, Projectile};
use crate::random::{random_colour, random_float};
use crate::tilemap::{
    MapOffset, TileOffset, HALF_HEIGHT, HALF_WIDTH, RENDERED_HEIGHT, RENDERED_WIDTH, TILE_SIZE,
};
use bevy_rand::prelude::{GlobalEntropy, WyRand};

/// A plugin responsible for managing player-related logic.
///
/// This plugin registers systems for player spawning, input handling (movement and shooting),
/// and camera scrolling, ensuring they run only when the game is in the `Playing` state.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    // Player input systems are grouped in the `Input` set from MovementSystems.
                    handle_player_input.in_set(MovementSystems::Input),
                    handle_shoot.in_set(MovementSystems::Input),
                    // Camera scrolling logic runs after the player's position has been updated.
                    adjust_scroll_for_buffer.in_set(MovementSystems::AdjustScroll),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// A marker component used to identify the player entity.
#[derive(Component)]
pub struct Player;

/// The base speed multiplier for player and projectile movement.
pub const DEFAULT_PLAYER_SPEED: f32 = 50.0;
/// Defines the size of the "camera deadzone" in tiles. The camera will not scroll
/// until the player moves beyond this buffer area from the center of the screen.
const BUFFER_TILES: Vec2 = Vec2::new(8.0, 8.0);

/// Spawns the player entity at a random, valid (non-wall) location on the map.
///
/// This system runs once when entering the `GameState::Playing` state. It also
/// calculates the initial map and tile offsets to center the camera on the
/// newly spawned player.
fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
    map_data: Res<MapData>,
    mut map_offset: ResMut<MapOffset>,
    mut tile_offset: ResMut<TileOffset>,
    mut reservations: ResMut<GridReservations>,
) {
    let width = map_data.width as i32;
    let height = map_data.height as i32;
    let mut mx: i32;
    let mut my: i32;

    // Loop until a valid, non-wall starting position is found.
    loop {
        mx = (random_float(&mut rng) * width as f32) as i32;
        my = (random_float(&mut rng) * height as f32) as i32;
        let flipped_y = (height - 1 - my) as u32; // Map data is stored with Y-axis flipped.
        let idx = (flipped_y * map_data.width + mx as u32) as usize;
        if let Some(&is_wall) = map_data.is_wall.get(idx) {
            if !is_wall {
                break; // Found a valid spot.
            }
        }
    }

    // Calculate the initial integer-based map offset to position the player near the center of the view.
    // This is clamped to ensure the view doesn't go outside the map boundaries.
    let ox =
        ((mx as f32 - HALF_WIDTH).floor() as i32).clamp(0, (width - RENDERED_WIDTH as i32).max(0));
    let oy = ((my as f32 - HALF_HEIGHT).floor() as i32)
        .clamp(0, (height - RENDERED_HEIGHT as i32).max(0));
    map_offset.0 = IVec2::new(ox, oy);

    // Calculate the fractional (sub-tile) offset needed for smooth scrolling.
    let frac_x = mx as f32 - ox as f32 - HALF_WIDTH;
    let frac_y = my as f32 - oy as f32 - HALF_HEIGHT;
    tile_offset.0 = Vec2::new(-frac_x * TILE_SIZE, -frac_y * TILE_SIZE);

    // Spawn the player entity with all its necessary components.
    let player_entity = commands
        .spawn((
            Sprite {
                color: Color::WHITE,
                image: game_assets.player_texture.clone(),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0), // Initial position is centered, adjusted by GridMover.
            Player,
            GridMover {
                grid_pos: IVec2::new(mx, my),
                direction: IVec2::ZERO,
                progress: 0.0,
                speed: 20.0 * DEFAULT_PLAYER_SPEED,
            },
            IntendedDirection(IVec2::ZERO),
            GameEntity, // Marker for cleanup when returning to the title screen.
            Collider {
                size: Vec2::splat(TILE_SIZE * 0.5), // A smaller collider than the tile size.
            },
            GridReserver, // Add the reserver component
        ))
        .id();

    // Make the initial reservation for the player's starting cell.
    reservations.0.insert(IVec2::new(mx, my), player_entity);
}

/// Reads keyboard input (W, A, S, D) to set the player's intended direction of movement.
///
/// This system updates the `IntendedDirection` component, which is then used by the
/// `update_grid_movement` system to control the `GridMover`.
fn handle_player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut IntendedDirection, With<Player>>,
) {
    if let Ok(mut intended) = query.single_mut() {
        let mut dx = 0i32;
        if keys.pressed(KeyCode::KeyA) {
            dx -= 1;
        }
        if keys.pressed(KeyCode::KeyD) {
            dx += 1;
        }
        let mut dy = 0i32;
        if keys.pressed(KeyCode::KeyS) {
            dy -= 1;
        }
        if keys.pressed(KeyCode::KeyW) {
            dy += 1;
        }
        intended.0 = IVec2::new(dx, dy);
    }
}

/// Handles the player's shooting action based on keyboard input.
///
/// When the Space key is pressed, this system spawns a projectile entity.
/// The projectile is spawned one tile ahead of the player in their current
/// intended direction of movement. No projectile is fired if the player is stationary
/// or aiming at a wall.
fn handle_shoot(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut rng: GlobalEntropy<WyRand>,
    game_assets: Res<GameAssets>,
    query: Query<(&GridMover, &IntendedDirection), With<Player>>,
    map_data: Res<MapData>,
) {
    // Check for the shoot button press.
    if keys.just_pressed(KeyCode::Space) {
        if let Ok((mover, intended)) = query.single() {
            info!("space pressed");
            // Only shoot if the player has a direction.
            if intended.0 != IVec2::ZERO {
                let dir = intended.0;
                let spawn_pos = mover.grid_pos + dir; // Spawn in the next tile over.

                // Prevent spawning a projectile inside a wall.
                if is_wall(spawn_pos, &map_data) {
                    return;
                }
                let color = random_colour(&mut rng, &game_assets);

                // Spawn the projectile entity.
                commands.spawn((
                    Sprite {
                        color,
                        image: game_assets.player_texture.clone(), // Uses player texture for now.
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    Projectile,
                    GridMover {
                        grid_pos: spawn_pos,
                        direction: IVec2::ZERO, // Initially stationary, will move on next frame.
                        progress: 0.0,
                        speed: 30.0 * DEFAULT_PLAYER_SPEED,
                    },
                    IntendedDirection(dir), // The projectile continues in the player's direction.
                    Bouncable { remaining: 3 }, // Can bounce off walls 3 times.
                    Collider {
                        size: Vec2::splat(TILE_SIZE * 0.5),
                    },
                    GameEntity,
                ));
                // Play the shooting sound effect.
                audio::play(&mut commands, game_assets.shoot_sfx.clone());
            }
        }
    }
}

/// Implements camera scrolling by adjusting map and tile offsets.
///
/// This function creates a "deadzone" or buffer around the center of the screen. The
/// map remains static until the player moves outside this buffer. Once the player crosses
/// the buffer boundary, the `tile_offset` is adjusted to smoothly scroll the world,
/// keeping the player within the buffer. When the `tile_offset` exceeds the size of a
/// full tile, it "wraps around," and the integer-based `map_offset` is updated. This
/// entire process is clamped to prevent the camera from scrolling past the map's edges.
fn adjust_scroll_for_buffer(
    query_player: Query<&Transform, With<Player>>,
    mut map_offset: ResMut<MapOffset>,
    mut tile_offset: ResMut<TileOffset>,
    map_data: Res<MapData>,
) {
    // Calculate the pixel dimensions of the central buffer zone.
    let half_buffer = BUFFER_TILES * TILE_SIZE / 2.0;

    if let Ok(player_tr) = query_player.single() {
        let p = player_tr.translation.xy();
        let mut delta = Vec2::ZERO;

        // Check if the player's screen position has exceeded the buffer boundaries.
        if p.x > half_buffer.x {
            delta.x = p.x - half_buffer.x;
        } else if p.x < -half_buffer.x {
            delta.x = p.x + half_buffer.x;
        }
        if p.y > half_buffer.y {
            delta.y = p.y - half_buffer.y;
        } else if p.y < -half_buffer.y {
            delta.y = p.y + half_buffer.y;
        }

        // If the player is outside the buffer, adjust the tile offset to scroll the map.
        if delta != Vec2::ZERO {
            tile_offset.0 -= delta;
        }
    }

    // Define map and view dimensions for clamping.
    let map_width_i = map_data.width as i32;
    let map_height_i = map_data.height as i32;
    let view_width = RENDERED_WIDTH as i32;
    let view_height = RENDERED_HEIGHT as i32;
    let max_map_x = (map_width_i - view_width).max(0);
    let max_map_y = (map_height_i - view_height).max(0);

    // This section handles the "wrapping" of the tile_offset into the map_offset
    // and clamps the final view position to the map boundaries.

    // Handle X-axis scrolling and clamping.
    let mut view_left = map_offset.0.x as f32 - tile_offset.0.x / TILE_SIZE;
    view_left = view_left.clamp(0.0, max_map_x as f32);
    map_offset.0.x = view_left.floor() as i32;
    tile_offset.0.x = -(view_left - map_offset.0.x as f32) * TILE_SIZE;

    // Handle Y-axis scrolling and clamping.
    let mut view_top = map_offset.0.y as f32 - tile_offset.0.y / TILE_SIZE;
    view_top = view_top.clamp(0.0, max_map_y as f32);
    map_offset.0.y = view_top.floor() as i32;
    tile_offset.0.y = -(view_top - map_offset.0.y as f32) * TILE_SIZE;
}
