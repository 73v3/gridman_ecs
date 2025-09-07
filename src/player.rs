// player.rs
use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::audio;
use crate::collider::Collider;
use crate::components::{GameEntity, GameState};
use crate::grid_movement::{is_wall, GridMover, IntendedDirection, MovementSystems};
use crate::map::MapData;
use crate::projectile::{Bouncable, Projectile};
use crate::random::{random_colour, random_float};
use crate::tilemap::{
    MapOffset, TileOffset, HALF_HEIGHT, HALF_WIDTH, RENDERED_HEIGHT, RENDERED_WIDTH, TILE_SIZE,
};
use bevy_rand::prelude::{GlobalEntropy, WyRand};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    handle_player_input.in_set(MovementSystems::Input),
                    handle_shoot.in_set(MovementSystems::Input),
                    adjust_scroll_for_buffer.in_set(MovementSystems::AdjustScroll),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Player;

const DEFAULT_PLAYER_SPEED: f32 = 50.0;
const BUFFER_TILES: Vec2 = Vec2::new(4.0, 4.0);

fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
    map_data: Res<MapData>,
    mut map_offset: ResMut<MapOffset>,
    mut tile_offset: ResMut<TileOffset>,
) {
    let width = map_data.width as i32;
    let height = map_data.height as i32;
    let mut mx: i32;
    let mut my: i32;
    loop {
        mx = (random_float(&mut rng) * width as f32) as i32;
        my = (random_float(&mut rng) * height as f32) as i32;
        let flipped_y = (height - 1 - my) as u32;
        let idx = (flipped_y * map_data.width + mx as u32) as usize;
        if let Some(&is_wall) = map_data.is_wall.get(idx) {
            if !is_wall {
                break;
            }
        }
    }

    let ox =
        ((mx as f32 - HALF_WIDTH).floor() as i32).clamp(0, (width - RENDERED_WIDTH as i32).max(0));
    let oy = ((my as f32 - HALF_HEIGHT).floor() as i32)
        .clamp(0, (height - RENDERED_HEIGHT as i32).max(0));
    map_offset.0 = IVec2::new(ox, oy);

    let frac_x = mx as f32 - ox as f32 - HALF_WIDTH;
    let frac_y = my as f32 - oy as f32 - HALF_HEIGHT;
    tile_offset.0 = Vec2::new(-frac_x * TILE_SIZE, -frac_y * TILE_SIZE);

    commands.spawn((
        Sprite {
            color: Color::WHITE,
            image: game_assets.player_texture.clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player,
        GridMover {
            grid_pos: IVec2::new(mx, my),
            direction: IVec2::ZERO,
            progress: 0.0,
            speed: 20.0 * DEFAULT_PLAYER_SPEED,
        },
        IntendedDirection(IVec2::ZERO),
        GameEntity,
        Collider {
            size: Vec2::splat(TILE_SIZE * 0.5),
        },
    ));
}

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

fn handle_shoot(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut rng: GlobalEntropy<WyRand>,
    game_assets: Res<GameAssets>,
    query: Query<(&GridMover, &IntendedDirection), With<Player>>,
    map_data: Res<MapData>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if let Ok((mover, intended)) = query.single() {
            info!("space pressed");
            if mover.progress == 0.0 && intended.0 != IVec2::ZERO {
                let dir = intended.0;
                let next_tile = mover.grid_pos + dir;
                if is_wall(next_tile, &map_data) {
                    return;
                }
                let color = random_colour(&mut rng, &game_assets);
                commands.spawn((
                    Sprite {
                        color,
                        image: game_assets.player_texture.clone(),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    Projectile,
                    GridMover {
                        grid_pos: mover.grid_pos,
                        direction: IVec2::ZERO,
                        progress: 0.0,
                        speed: 30.0 * DEFAULT_PLAYER_SPEED,
                    },
                    IntendedDirection(dir),
                    Bouncable { remaining: 3 },
                    Collider {
                        size: Vec2::splat(TILE_SIZE * 0.5),
                    },
                    GameEntity,
                ));
                audio::play(&mut commands, game_assets.shoot_sfx.clone());
            }
        }
    }
}

fn adjust_scroll_for_buffer(
    mut query_player: Query<&mut Transform, With<Player>>,
    mut map_offset: ResMut<MapOffset>,
    mut tile_offset: ResMut<TileOffset>,
    map_data: Res<MapData>,
) {
    let half_buffer = BUFFER_TILES * TILE_SIZE / 2.0;
    if let Ok(mut player_tr) = query_player.single_mut() {
        let p = player_tr.translation.xy();
        let mut delta = Vec2::ZERO;
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
        if delta != Vec2::ZERO {
            tile_offset.0 -= delta;
            player_tr.translation.x -= delta.x;
            player_tr.translation.y -= delta.y;
        }
    }

    let map_width_i = map_data.width as i32;
    let map_height_i = map_data.height as i32;
    let view_width = RENDERED_WIDTH as i32;
    let view_height = RENDERED_HEIGHT as i32;
    let max_map_x = (map_width_i - view_width).max(0);
    let max_map_y = (map_height_i - view_height).max(0);

    let mut view_left = map_offset.0.x as f32 - tile_offset.0.x / TILE_SIZE;
    view_left = view_left.clamp(0.0, max_map_x as f32);
    map_offset.0.x = view_left.floor() as i32;
    tile_offset.0.x = -(view_left - map_offset.0.x as f32) * TILE_SIZE;

    let mut view_top = map_offset.0.y as f32 - tile_offset.0.y / TILE_SIZE;
    view_top = view_top.clamp(0.0, max_map_y as f32);
    map_offset.0.y = view_top.floor() as i32;
    tile_offset.0.y = -(view_top - map_offset.0.y as f32) * TILE_SIZE;
}
