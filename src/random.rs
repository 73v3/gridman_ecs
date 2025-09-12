// src/random.rs

use bevy::prelude::*;
use bevy_rand::prelude::{EntropyPlugin, GlobalEntropy, WyRand};

use crate::assets::GameAssets;
use rand_core::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};

/// Plugin for handling random number generation with WyRand
#[derive(Debug, Clone, Copy, Default)]
pub struct RandomPlugin;

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut App) {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;

        // The `with_seed` function expects a byte array.
        // We convert the u64 seed to a little-endian byte array.
        app.add_plugins(EntropyPlugin::<WyRand>::with_seed(seed.to_le_bytes()));
    }
}

/// Returns a random float between 0.0 and 1.0
pub fn random_float(rng: &mut GlobalEntropy<WyRand>) -> f32 {
    (rng.next_u32() as f32) / (u32::MAX as f32)
}

/// Returns a random color from the GameAssets palette
pub fn random_colour(rng: &mut GlobalEntropy<WyRand>, game_assets: &Res<GameAssets>) -> Color {
    let palette = &game_assets.palette;
    let index = (random_float(rng) * palette.colors.len() as f32) as usize;
    palette.colors[index]
}

/// Returns a random color from the GameAssets palette, excluding the specified color.
/// If the palette is empty or only contains the excluded color, returns Color::WHITE.
pub fn random_colour_except(
    rng: &mut GlobalEntropy<WyRand>,
    game_assets: &Res<GameAssets>,
    except_colour: Color,
) -> Color {
    let palette = &game_assets.palette;

    // Handle empty palette
    if palette.colors.is_empty() {
        return Color::WHITE;
    }

    // Find the index of the excluded color, if it exists
    let exclude_index = palette.colors.iter().position(|&c| c == except_colour);

    // If palette has only one color and it's the excluded one, return fallback
    if palette.colors.len() == 1 && exclude_index == Some(0) {
        return Color::WHITE;
    }

    // Calculate the range for random selection (subtract 1 if excluding a color)
    let range = palette.colors.len() - exclude_index.map_or(0, |_| 1);
    let idx = (random_float(rng) * range as f32) as usize;

    // Adjust index to skip the excluded color
    let final_index = match exclude_index {
        Some(ex) if idx >= ex => idx + 1,
        _ => idx,
    };

    palette.colors[final_index]
}
