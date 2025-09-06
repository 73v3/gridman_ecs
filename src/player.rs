use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState, Speed, Velocity};
use crate::map::MapData;
use crate::random::random_float;
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
                (handle_player_input, adjust_scroll_for_buffer)
                    .chain()
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
        Velocity {
            velocity: Vec2::ZERO,
        },
        Speed {
            value: DEFAULT_PLAYER_SPEED,
        },
        GameEntity,
    ));
}

fn handle_player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &Speed), With<Player>>,
) {
    for (mut vel, speed) in &mut query {
        let mut dir = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }
        vel.velocity = if dir != Vec2::ZERO {
            dir.normalize() * speed.value
        } else {
            Vec2::ZERO
        };
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
            tile_offset.0.x -= delta.x;
            tile_offset.0.y -= delta.y;
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
