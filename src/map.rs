use crate::components::GameState;
use crate::random::random_float;
use bevy::prelude::*;
use bevy_rand::prelude::{GlobalEntropy, WyRand};

pub const MAP_WIDTH: u32 = 80;
pub const MAP_HEIGHT: u32 = 80;
pub const NUM_WALKS: usize = 128;
pub const BORDER_WIDTH: i32 = 2;

#[derive(Resource)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
    pub is_wall: Vec<bool>,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), generate_map);
    }
}

// Generates a procedural map using random walks to carve two-tile-wide paths through an initial wall-filled grid.
// This system runs when entering the Playing state to create a new map for each game session.
pub fn generate_map(mut commands: Commands, mut rng: GlobalEntropy<WyRand>) {
    let width = MAP_WIDTH;
    let height = MAP_HEIGHT;
    let mut is_wall = vec![true; (width * height) as usize];

    let min_coord = BORDER_WIDTH; // Start from 2 to leave 0 and 1 as walls
    let max_coord = width as i32 - BORDER_WIDTH; // Up to 77 to leave 78 and 79 as walls

    let directions = vec![
        IVec2::new(0, 1),  // North
        IVec2::new(0, -1), // South
        IVec2::new(1, 0),  // East
        IVec2::new(-1, 0), // West
    ];

    for _ in 0..NUM_WALKS {
        // Choose a starting position that allows both primary and secondary tiles to be valid
        let mut x;
        let mut y;
        loop {
            x = (random_float(&mut rng) * (max_coord - min_coord + 1) as f32).floor() as i32
                + min_coord;
            y = (random_float(&mut rng) * (max_coord - min_coord + 1) as f32).floor() as i32
                + min_coord;
            // Ensure secondary tile (x+1 or y+1) is also within bounds
            if x + 1 < max_coord && y + 1 < max_coord {
                break;
            }
        }
        let mut pos = IVec2::new(x, y);

        // First leg of the walk
        let dir_idx = (random_float(&mut rng) * 4.0).floor() as usize;
        let mut dir = directions[dir_idx];
        // Halve the walk length to account for double tile carving
        let n = (random_float(&mut rng) * (width - 1) as f32 / 2.0).floor() as i32 + 1;
        for _ in 0..n {
            let next_pos = pos + dir;
            // Check if primary tile is within bounds
            if next_pos.x < min_coord
                || next_pos.x >= max_coord
                || next_pos.y < min_coord
                || next_pos.y >= max_coord
            {
                break;
            }
            set_floor(&mut is_wall, pos, dir, width, height);
            pos = next_pos;
        }

        // Turn 90 degrees
        let clockwise = random_float(&mut rng) < 0.5;
        dir = if clockwise {
            IVec2::new(dir.y, -dir.x) // Clockwise: (x,y) -> (y,-x)
        } else {
            IVec2::new(-dir.y, dir.x) // Counterclockwise: (x,y) -> (-y,x)
        };

        // Second leg of the walk
        let m = (random_float(&mut rng) * (height - 1) as f32 / 2.0).floor() as i32 + 1;
        for _ in 0..m {
            let next_pos = pos + dir;
            if next_pos.x < min_coord
                || next_pos.x >= max_coord
                || next_pos.y < min_coord
                || next_pos.y >= max_coord
            {
                break;
            }
            set_floor(&mut is_wall, pos, dir, width, height);
            pos = next_pos;
        }
    }

    commands.insert_resource(MapData {
        width,
        height,
        is_wall,
    });
}

// Sets two adjacent tiles to floor (not wall) based on the direction of movement, respecting the flipped y-indexing.
fn set_floor(is_wall: &mut Vec<bool>, pos: IVec2, dir: IVec2, width: u32, height: u32) {
    let x = pos.x as usize;
    let y = pos.y as usize;
    let flipped_y = (height - 1 - y as u32) as usize;
    let idx = flipped_y * width as usize + x;

    // Check if primary tile is within bounds and not in border
    let min_coord = BORDER_WIDTH;
    let max_coord = width as i32 - BORDER_WIDTH;
    if pos.x < min_coord || pos.x >= max_coord || pos.y < min_coord || pos.y >= max_coord {
        return; // Skip if primary tile is in border or out of bounds
    }
    if idx < is_wall.len() {
        is_wall[idx] = false;
    }

    // Determine secondary tile based on direction
    let (sec_x, sec_y) = if dir.y != 0 {
        (pos.x + 1, pos.y) // North/South: pair with tile to the right
    } else {
        (pos.x, pos.y + 1) // East/West: pair with tile above
    };

    // Check if secondary tile is within bounds and not in border
    if sec_x >= min_coord && sec_x < max_coord && sec_y >= min_coord && sec_y < max_coord {
        let sec_flipped_y = (height - 1 - sec_y as u32) as usize;
        let sec_idx = sec_flipped_y * width as usize + sec_x as usize;
        if sec_idx < is_wall.len() {
            is_wall[sec_idx] = false;
        }
    }
}
