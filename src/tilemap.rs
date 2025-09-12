// tilemap.rs
use bevy::prelude::*;
use bevy::sprite::Sprite;
use bevy_rand::prelude::{GlobalEntropy, WyRand};

use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState};
use crate::map::{generate_map, MapData};
use crate::random::random_colour_except;

pub const TILE_SIZE: f32 = 64.0;
pub const RENDERED_WIDTH: usize = 36;
pub const RENDERED_HEIGHT: usize = 28;
pub const HALF_WIDTH: f32 = (RENDERED_WIDTH as f32 - 1.0) / 2.0;
pub const HALF_HEIGHT: f32 = (RENDERED_HEIGHT as f32 - 1.0) / 2.0;
/// Defines the size of one side of a checkerboard square, in tiles.
pub const CHECKER_SIZE: u32 = 4;

pub const WALL_COLOUR_INDEX: usize = 13; // colour of wall in terms of asset palette index

#[derive(Resource)]
pub struct MapOffset(pub IVec2);

#[derive(Resource)]
pub struct TileOffset(pub Vec2);

/// A resource to hold the two darkened, randomized colors for the floor pattern.
#[derive(Resource)]
pub struct FloorPalette {
    pub color_a: Color,
    pub color_b: Color,
}

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
                (
                    setup_initial_offset,
                    setup_floor_palette, // Create the random palette
                    spawn_tilemap,
                )
                    .chain()
                    .after(generate_map),
            )
            .add_systems(
                Update,
                ((update_tile_positions, update_tile_colors)
                    .run_if(resource_changed::<MapOffset>.or(resource_changed::<TileOffset>)),)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// A new system that runs once to create and store the floor palette.
/// It picks two random colors, excluding the wall color, darkens them, and inserts them as a resource.
fn setup_floor_palette(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
) {
    // Get the wall color to exclude
    let wall_color = game_assets.palette.colors[WALL_COLOUR_INDEX];

    // Pick two random different colors, excluding the wall color
    let mut color_a = random_colour_except(&mut rng, &game_assets, wall_color);
    let mut color_b = random_colour_except(&mut rng, &game_assets, wall_color);
    while color_a == color_b {
        color_b = random_colour_except(&mut rng, &game_assets, wall_color);
    }

    // Darken them
    let darken_factor = 0.25;
    color_a = darken(color_a, darken_factor);
    color_b = darken(color_b, darken_factor);

    // Insert them into a resource
    commands.insert_resource(FloorPalette { color_a, color_b });
}

fn darken(c: Color, darken_factor: f32) -> Color {
    match c {
        Color::Srgba(mut srgba) => {
            srgba.red *= darken_factor;
            srgba.green *= darken_factor;
            srgba.blue *= darken_factor;
            Color::Srgba(srgba)
        }
        _ => c,
    }
}

// Center map in viewport
fn setup_initial_offset(map_data: Res<MapData>, mut map_offset: ResMut<MapOffset>) {
    let view_w = RENDERED_WIDTH as i32;
    let view_h = RENDERED_HEIGHT as i32;
    let map_w = map_data.width as i32;
    let map_h = map_data.height as i32;
    map_offset.0.x = ((map_w - view_w) / 2).max(0);
    map_offset.0.y = ((map_h - view_h) / 2).max(0);
}

// Spawns the viewable section of the tilemap, with each visible tile being an individual sprite entity
fn spawn_tilemap(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    map_data: Res<MapData>,
    map_offset: Res<MapOffset>,
    floor_palette: Res<FloorPalette>, // Get the newly created floor palette
) {
    let wall_texture = game_assets.wall_texture.clone();

    for gx in 0..RENDERED_WIDTH {
        for gy in 0..RENDERED_HEIGHT {
            let base_x = (gx as f32 - HALF_WIDTH) * TILE_SIZE;
            let base_y = (gy as f32 - HALF_HEIGHT) * TILE_SIZE;
            let base_pos = Vec2::new(base_x, base_y);

            let grid_pos = IVec2::new(gx as i32, gy as i32);
            let map_pos = grid_pos + map_offset.0;
            // Pass the palette to the color logic function
            let color = get_tile_color(map_pos, &game_assets, &map_data, &floor_palette);

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

/// Updated to determine tile color based on walls and the new checkerboard floor.
fn get_tile_color(
    map_pos: IVec2,
    game_assets: &GameAssets,
    map_data: &MapData,
    floor_palette: &FloorPalette,
) -> Color {
    // First, check if the position is within the map's boundaries.
    // If not, return a transparent color to avoid drawing outside the map area.
    if map_pos.x < 0
        || map_pos.y < 0
        || map_pos.x >= map_data.width as i32
        || map_pos.y >= map_data.height as i32
    {
        return Color::NONE;
    }

    // Determine if the current tile is a wall.
    let x = map_pos.x as u32;
    let y = map_pos.y as u32;
    let flipped_y = map_data.height - 1 - y;
    let idx = (flipped_y * map_data.width + x) as usize;
    let is_wall = map_data.is_wall.get(idx).copied().unwrap_or(false);

    if is_wall {
        // It's a wall, so calculate its color based on its position.
        let index = WALL_COLOUR_INDEX; // uncomment if you want walls to use entire palette -> ((map_pos.x.abs() + map_pos.y.abs()) as usize) % game_assets.palette.colors.len();
        game_assets.palette.colors[index]
    } else {
        // It's a floor tile, so apply the checkerboard pattern.
        // Use Euclidean division to handle potential negative coordinates gracefully.
        let checker_x = map_pos.x.div_euclid(CHECKER_SIZE as i32);
        let checker_y = map_pos.y.div_euclid(CHECKER_SIZE as i32);
        if (checker_x + checker_y) % 2 == 0 {
            floor_palette.color_a
        } else {
            floor_palette.color_b
        }
    }
}

/// Updated to pass the FloorPalette resource to the color logic.
fn update_tile_colors(
    map_offset: Res<MapOffset>,
    game_assets: Res<GameAssets>,
    map_data: Res<MapData>,
    floor_palette: Res<FloorPalette>, // Get the floor palette
    mut query: Query<(&Tile, &mut Sprite)>,
) {
    for (tile, mut sprite) in query.iter_mut() {
        let map_pos = map_offset.0 + tile.grid_pos;
        // Pass the palette to the color logic function
        sprite.color = get_tile_color(map_pos, &game_assets, &map_data, &floor_palette);
    }
}
