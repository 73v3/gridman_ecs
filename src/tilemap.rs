// tilemap.rs
use bevy::prelude::*;
use bevy::sprite::Sprite;

use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState};
use crate::map::MapData;

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
            .add_systems(
                OnEnter(GameState::Playing),
                (setup_initial_offset, spawn_tilemap).chain(),
            )
            .add_systems(
                Update,
                (
                    handle_scroll_input,
                    (update_tile_positions, update_tile_colors)
                        .run_if(resource_changed::<MapOffset>.or(resource_changed::<TileOffset>)),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// center map in viewport
fn setup_initial_offset(map_data: Res<MapData>, mut map_offset: ResMut<MapOffset>) {
    let view_w = RENDERED_WIDTH as i32;
    let view_h = RENDERED_HEIGHT as i32;
    let map_w = map_data.width as i32;
    let map_h = map_data.height as i32;
    map_offset.0.x = ((map_w - view_w) / 2).max(0);
    map_offset.0.y = ((map_h - view_h) / 2).max(0);
}

// spawns the viewable section of the tilemap, with each visible tile being an individual sprite entity
fn spawn_tilemap(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    map_data: Res<MapData>,
    map_offset: Res<MapOffset>,
) {
    let wall_texture = game_assets.wall_texture.clone();

    for gx in 0..RENDERED_WIDTH {
        for gy in 0..RENDERED_HEIGHT {
            let base_x = (gx as f32 - HALF_WIDTH) * TILE_SIZE;
            let base_y = (gy as f32 - HALF_HEIGHT) * TILE_SIZE;
            let base_pos = Vec2::new(base_x, base_y);

            let grid_pos = IVec2::new(gx as i32, gy as i32);
            let map_pos = grid_pos + map_offset.0;
            let color = get_tile_color(map_pos, &game_assets, &map_data);

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

// boundary-constrained scrolling via the arrow keys
fn handle_scroll_input(
    mut tile_offset: ResMut<TileOffset>,
    mut map_offset: ResMut<MapOffset>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map_data: Res<MapData>,
) {
    let speed = 800.0; // pixels per second, adjust as needed
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

    let map_width_i = map_data.width as i32;
    let map_height_i = map_data.height as i32;
    let view_width = RENDERED_WIDTH as i32;
    let view_height = RENDERED_HEIGHT as i32;
    let max_map_x = (map_width_i - view_width).max(0);
    let max_map_y = (map_height_i - view_height).max(0);

    // Clamp x boundaries
    let mut view_left = map_offset.0.x as f32 - tile_offset.0.x / TILE_SIZE;
    view_left = view_left.clamp(0.0, max_map_x as f32);
    map_offset.0.x = view_left.floor() as i32;
    tile_offset.0.x = -(view_left - map_offset.0.x as f32) * TILE_SIZE;

    // Clamp y boundaries
    let mut view_top = map_offset.0.y as f32 - tile_offset.0.y / TILE_SIZE;
    view_top = view_top.clamp(0.0, max_map_y as f32);
    map_offset.0.y = view_top.floor() as i32;
    tile_offset.0.y = -(view_top - map_offset.0.y as f32) * TILE_SIZE;
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

fn get_tile_color(map_pos: IVec2, game_assets: &GameAssets, map_data: &MapData) -> Color {
    let x = map_pos.x as u32;
    let y = map_pos.y as u32;
    if x >= map_data.width || y >= map_data.height {
        return Color::NONE;
    }
    let flipped_y = map_data.height - 1 - y;
    let idx = (flipped_y * map_data.width + x) as usize;
    if map_data.is_wall.get(idx).copied().unwrap_or(false) {
        let index =
            ((map_pos.x.abs() + map_pos.y.abs()) as usize) % game_assets.palette.colors.len();
        game_assets.palette.colors[index]
    } else {
        Color::NONE
    }
}

fn update_tile_colors(
    map_offset: Res<MapOffset>,
    game_assets: Res<GameAssets>,
    map_data: Res<MapData>,
    mut query: Query<(&Tile, &mut Sprite)>,
) {
    for (tile, mut sprite) in query.iter_mut() {
        let map_pos = map_offset.0 + tile.grid_pos;
        sprite.color = get_tile_color(map_pos, &game_assets, &map_data);
    }
}
