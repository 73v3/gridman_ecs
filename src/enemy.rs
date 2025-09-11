// enemy.rs

//! Manages enemy spawning, AI, and behavior.

use bevy::prelude::*;
use bevy_rand::prelude::{GlobalEntropy, WyRand};

use crate::assets::GameAssets;
use crate::collider::Collider;
use crate::components::{GameEntity, GameState};
use crate::grid_movement::{self, GridMover, IntendedDirection, MovementSystems};
use crate::grid_reservation::{GridReservations, GridReserver};
use crate::map::MapData;
use crate::player::{spawn_player, Player, DEFAULT_PLAYER_SPEED};
use crate::random::{random_colour, random_float};
use crate::tilemap::TILE_SIZE;

const NUM_LEFT_TURNERS: u32 = 150;
const NUM_RIGHT_TURNERS: u32 = NUM_LEFT_TURNERS;

const DEFAULT_ENEMY_SPEED: f32 = 0.5 * DEFAULT_PLAYER_SPEED;

/// A plugin for all enemy-related logic.
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Title), setup_enemy_colors)
            .add_systems(
                OnEnter(GameState::Playing),
                spawn_enemies.after(spawn_player),
            )
            .configure_sets(
                Update,
                // The AI systems must run before the movement system to avoid a 1-frame delay.
                EnemyMovementAI.before(MovementSystems::UpdateMover),
            )
            .add_systems(
                Update,
                (update_left_turners, update_right_turners)
                    .in_set(EnemyMovementAI)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// A SystemSet for enemy AI logic to ensure it runs before movement is executed.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct EnemyMovementAI;

/// A marker component for any enemy entity.
#[derive(Component)]
pub struct Enemy;

/// A stateful component for enemies that prefer turning left.
#[derive(Component)]
pub struct LeftTurner {
    /// The last direction the enemy was intentionally moving.
    /// This is crucial for making turn decisions after being stopped by a collision.
    pub last_known_direction: IVec2,
}

/// A stateful component for enemies that prefer turning right.
#[derive(Component)]
pub struct RightTurner {
    /// The last direction the enemy was intentionally moving.
    pub last_known_direction: IVec2,
}

/// A resource to store the globally chosen colors for each enemy type.
#[derive(Resource)]
pub struct EnemyColors {
    pub left_turner: Color,
    pub right_turner: Color,
}

/// Runs once to select and store the colors for enemies.
fn setup_enemy_colors(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
) {
    let color_a = random_colour(&mut rng, &game_assets);
    let mut color_b = random_colour(&mut rng, &game_assets);
    // Ensure the two colors are different.
    while color_a == color_b {
        color_b = random_colour(&mut rng, &game_assets);
    }
    commands.insert_resource(EnemyColors {
        left_turner: color_a,
        right_turner: color_b,
    });
}

/// Spawns all initial enemies in random, valid locations.
fn spawn_enemies(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
    map_data: Res<MapData>,
    mut reservations: ResMut<GridReservations>,
    enemy_colors: Res<EnemyColors>,
    player_query: Query<&GridMover, With<Player>>,
) {
    let player_pos = player_query.single().unwrap().grid_pos;
    info!("Spawning enemies, player position: {:?}", player_pos);
    let valid_directions = [
        IVec2::new(0, 1),
        IVec2::new(0, -1),
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
    ];

    // Spawn LeftTurners
    for _ in 0..NUM_LEFT_TURNERS {
        let (spawn_pos, start_dir) = find_valid_spawn(
            &mut rng,
            &map_data,
            &reservations,
            &valid_directions,
            player_pos,
        );

        let entity = commands
            .spawn((
                Sprite {
                    color: enemy_colors.left_turner,
                    image: game_assets.enemy_texture.clone(),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.9),
                Enemy,
                GridMover {
                    grid_pos: spawn_pos,
                    direction: IVec2::ZERO,
                    progress: 0.0,
                    speed: DEFAULT_ENEMY_SPEED,
                },
                IntendedDirection(start_dir),
                LeftTurner {
                    last_known_direction: start_dir,
                },
                GridReserver,
                Collider {
                    size: Vec2::splat(TILE_SIZE * 0.5),
                },
                GameEntity,
            ))
            .id();
        reservations.0.insert(spawn_pos, entity);
    }

    // Spawn RightTurners
    for _ in 0..NUM_RIGHT_TURNERS {
        let (spawn_pos, start_dir) = find_valid_spawn(
            &mut rng,
            &map_data,
            &reservations,
            &valid_directions,
            player_pos,
        );

        let entity = commands
            .spawn((
                Sprite {
                    color: enemy_colors.right_turner,
                    image: game_assets.enemy_texture.clone(),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.9),
                Enemy,
                GridMover {
                    grid_pos: spawn_pos,
                    direction: IVec2::ZERO,
                    progress: 0.0,
                    speed: DEFAULT_ENEMY_SPEED,
                },
                IntendedDirection(start_dir),
                RightTurner {
                    last_known_direction: start_dir,
                },
                GridReserver,
                Collider {
                    size: Vec2::splat(TILE_SIZE * 0.5),
                },
                GameEntity,
            ))
            .id();
        reservations.0.insert(spawn_pos, entity);
    }
}

/// The AI system for LeftTurner enemies.
/// It decides on a new direction when the current path is blocked.
fn update_left_turners(
    mut query: Query<(Entity, &mut IntendedDirection, &GridMover, &mut LeftTurner)>,
    reservations: Res<GridReservations>,
    map_data: Res<MapData>,
) {
    for (entity, mut intended, mover, mut turner) in &mut query {
        // If the entity is moving, update its last known direction and do nothing else.
        if intended.0 != IVec2::ZERO {
            turner.last_known_direction = intended.0;
            continue;
        }

        // The entity has been stopped. Decide where to go next based on its last direction.
        let forward_dir = turner.last_known_direction;
        let current_pos = mover.grid_pos;

        // Priority: Left, Right, Back.
        let left_dir = IVec2::new(forward_dir.y, -forward_dir.x);
        let right_dir = IVec2::new(-forward_dir.y, forward_dir.x);
        let back_dir = -forward_dir;

        let new_dir = if !is_blocked(current_pos + left_dir, entity, &reservations, &map_data) {
            left_dir
        } else if !is_blocked(current_pos + right_dir, entity, &reservations, &map_data) {
            right_dir
        } else {
            back_dir
        };

        intended.0 = new_dir;
        turner.last_known_direction = new_dir;
    }
}

/// The AI system for RightTurner enemies.
/// It decides on a new direction when the current path is blocked.
fn update_right_turners(
    mut query: Query<(Entity, &mut IntendedDirection, &GridMover, &mut RightTurner)>,
    reservations: Res<GridReservations>,
    map_data: Res<MapData>,
) {
    for (entity, mut intended, mover, mut turner) in &mut query {
        // If the entity is moving, update its last known direction and do nothing else.
        if intended.0 != IVec2::ZERO {
            turner.last_known_direction = intended.0;
            continue;
        }

        // The entity has been stopped. Decide where to go next based on its last direction.
        let forward_dir = turner.last_known_direction;
        let current_pos = mover.grid_pos;

        // Priority: Right, Left, Back.
        let right_dir = IVec2::new(-forward_dir.y, forward_dir.x);
        let left_dir = IVec2::new(forward_dir.y, -forward_dir.x);
        let back_dir = -forward_dir;

        let new_dir = if !is_blocked(current_pos + right_dir, entity, &reservations, &map_data) {
            right_dir
        } else if !is_blocked(current_pos + left_dir, entity, &reservations, &map_data) {
            left_dir
        } else {
            back_dir
        };

        intended.0 = new_dir;
        turner.last_known_direction = new_dir;
    }
}

/// Helper to check if a target grid cell is a wall or reserved by another entity.
fn is_blocked(
    target_pos: IVec2,
    self_entity: Entity,
    reservations: &GridReservations,
    map_data: &MapData,
) -> bool {
    if grid_movement::is_wall(target_pos, map_data) {
        return true;
    }
    if let Some(&occupant) = reservations.0.get(&target_pos) {
        // A tile is only blocked if another entity occupies it.
        if occupant != self_entity {
            return true;
        }
    }
    false
}

/// Finds a random, non-wall, non-reserved grid cell to spawn an entity, ensuring it's at least 32 cells away from the player using Euclidean distance.
fn find_valid_spawn(
    rng: &mut GlobalEntropy<WyRand>,
    map_data: &MapData,
    reservations: &GridReservations,
    directions: &[IVec2],
    player_pos: IVec2,
) -> (IVec2, IVec2) {
    let width = map_data.width as i32;
    let height = map_data.height as i32;
    const MIN_DIST_SQ: i64 = 32 * 32;

    loop {
        let x = (random_float(rng) * width as f32) as i32;
        let y = (random_float(rng) * height as f32) as i32;
        let pos = IVec2::new(x, y);

        let dx = (x - player_pos.x) as i64;
        let dy = (y - player_pos.y) as i64;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq >= MIN_DIST_SQ
            && !grid_movement::is_wall(pos, map_data)
            && !reservations.0.contains_key(&pos)
        {
            // Found a valid position. Now find a valid starting direction.
            let start_idx = (random_float(rng) * directions.len() as f32) as usize;
            for i in 0..directions.len() {
                let dir = directions[(start_idx + i) % directions.len()];
                if !grid_movement::is_wall(pos + dir, map_data) {
                    return (pos, dir);
                }
            }
            // If all directions are blocked, we'll loop and find a new spawn point.
        }
    }
}
