// tilemap.rs
use bevy::prelude::*;
use bevy::sprite::Sprite;

use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState};

const TILE_SIZE: f32 = 64.0;
const RENDERED_WIDTH: usize = 28;
const RENDERED_HEIGHT: usize = 22;
const HALF_WIDTH: f32 = (RENDERED_WIDTH as f32 - 1.0) / 2.0;
const HALF_HEIGHT: f32 = (RENDERED_HEIGHT as f32 - 1.0) / 2.0;

#[derive(Resource)]
pub struct MapOffset(pub IVec2);

#[derive(Resource)]
pub struct TileOffset(pub Vec2);

#[derive(Component)]
pub struct Tile {
    pub grid_pos: IVec2,
}

#[derive(Component)]
pub struct BasePosition(pub Vec2);

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapOffset(IVec2::ZERO))
            .insert_resource(TileOffset(Vec2::ZERO))
            .add_systems(OnEnter(GameState::Playing), spawn_tilemap)
            .add_systems(
                Update,
                (
                    handle_scroll_input,
                    update_tile_positions,
                    update_tile_colors, //.run_if(resource_changed::<MapOffset>()),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_tilemap(mut commands: Commands, game_assets: Res<GameAssets>) {
    let wall_texture = game_assets.wall_texture.clone();

    for gx in 0..RENDERED_WIDTH {
        for gy in 0..RENDERED_HEIGHT {
            let base_x = (gx as f32 - HALF_WIDTH) * TILE_SIZE;
            let base_y = (gy as f32 - HALF_HEIGHT) * TILE_SIZE;
            let base_pos = Vec2::new(base_x, base_y);

            let grid_pos = IVec2::new(gx as i32, gy as i32);
            let map_pos = grid_pos; // Initial map_offset is ZERO
            let color = get_tile_color(map_pos, &game_assets);

            commands.spawn((
                Sprite {
                    image: wall_texture.clone(),
                    color,
                    ..Default::default()
                },
                Transform::from_xyz(base_x, base_y, 0.0),
                Tile { grid_pos },
                BasePosition(base_pos),
                GameEntity,
            ));
        }
    }
}

fn handle_scroll_input(
    mut tile_offset: ResMut<TileOffset>,
    mut map_offset: ResMut<MapOffset>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let speed = 200.0; // pixels per second, adjust as needed
    let dt = time.delta_secs();

    if keys.pressed(KeyCode::ArrowRight) {
        tile_offset.0.x -= speed * dt;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        tile_offset.0.x += speed * dt;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        tile_offset.0.y -= speed * dt;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        tile_offset.0.y += speed * dt;
    }

    // Handle x wrapping
    if tile_offset.0.x < -TILE_SIZE {
        tile_offset.0.x += TILE_SIZE;
        map_offset.0.x += 1;
    } else if tile_offset.0.x > TILE_SIZE {
        tile_offset.0.x -= TILE_SIZE;
        map_offset.0.x -= 1;
    }

    // Handle y wrapping
    if tile_offset.0.y < -TILE_SIZE {
        tile_offset.0.y += TILE_SIZE;
        map_offset.0.y += 1;
    } else if tile_offset.0.y > TILE_SIZE {
        tile_offset.0.y -= TILE_SIZE;
        map_offset.0.y -= 1;
    }
}

fn update_tile_positions(
    tile_offset: Res<TileOffset>,
    mut query: Query<(&BasePosition, &mut Transform), With<Tile>>,
) {
    for (base_pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            base_pos.0.x + tile_offset.0.x,
            base_pos.0.y + tile_offset.0.y,
            0.0,
        );
    }
}

fn update_tile_colors(
    map_offset: Res<MapOffset>,
    game_assets: Res<GameAssets>,
    mut query: Query<(&Tile, &mut Sprite)>,
) {
    for (tile, mut sprite) in query.iter_mut() {
        let map_pos = map_offset.0 + tile.grid_pos;
        sprite.color = get_tile_color(map_pos, &game_assets);
    }
}

fn get_tile_color(map_pos: IVec2, game_assets: &GameAssets) -> Color {
    let is_wall = (map_pos.x % 5 == 0) || (map_pos.y % 5 == 0);
    if is_wall {
        let index =
            ((map_pos.x.abs() + map_pos.y.abs()) as usize) % game_assets.palette.colors.len();
        game_assets.palette.colors[index]
    } else {
        Color::NONE
    }
}
