// assets.rs
use crate::components::GameState;
use bevy::audio::AudioSource;
use bevy::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), load_assets);
    }
}

#[derive(Resource, Clone)]
pub struct Palette {
    pub colors: Vec<Color>,
}

#[derive(Resource)]
pub struct GameAssets {
    pub wall_texture: Handle<Image>,
    pub player_texture: Handle<Image>,
    pub reservation_texture: Handle<Image>,
    pub enemy_texture: Handle<Image>,
    pub explosion_texture: Handle<Image>,
    pub font: Handle<Font>,
    pub shoot_sfx: Handle<AudioSource>,
    pub explosion_sfx: Handle<AudioSource>,
    pub palette: Palette,
}

//use bevy::prelude::Color;

// Parses a hex color string (e.g., "#83769C" or "83769C") and returns a Color::Srgba
pub fn color_from_hex(hex: &str) -> Result<Color, &'static str> {
    // Remove optional '#' prefix
    let hex = hex.trim_start_matches('#');

    // Ensure the hex string is valid (6 or 8 characters for RGB or RGBA)
    if hex.len() != 6 && hex.len() != 8 {
        return Err("Hex string must be 6 (RGB) or 8 (RGBA) characters long");
    }

    // Parse the hex string into u8 values
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex value for red")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex value for green")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex value for blue")?;

    // Handle alpha (default to 255 if not provided)
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex value for alpha")?
    } else {
        255
    };

    Ok(Color::srgba_u8(r, g, b, a))
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let palette = Palette {
        // https://lospec.com/palette-list/sweetie-16 by GrafxKid
        colors: vec![
            color_from_hex("#1a1c2c").unwrap(),
            color_from_hex("#5d275d").unwrap(),
            color_from_hex("#b13e53").unwrap(),
            color_from_hex("#ef7d57").unwrap(),
            color_from_hex("#ffcd75").unwrap(),
            color_from_hex("#a7f070").unwrap(),
            color_from_hex("#38b764").unwrap(),
            color_from_hex("#257179").unwrap(),
            color_from_hex("#29366f").unwrap(),
            color_from_hex("#3b5dc9").unwrap(),
            color_from_hex("#41a6f6").unwrap(),
            color_from_hex("#73eff7").unwrap(),
            color_from_hex("#f4f4f4").unwrap(),
            color_from_hex("#94b0c2").unwrap(),
            color_from_hex("#566c86").unwrap(),
            color_from_hex("#333c57").unwrap(),
        ],
    };

    commands.insert_resource(GameAssets {
        wall_texture: asset_server.load("textures/wall.png"),
        player_texture: asset_server.load("textures/player.png"),
        reservation_texture: asset_server.load("textures/reservation.png"),
        enemy_texture: asset_server.load("textures/enemy.png"),
        explosion_texture: asset_server.load("textures/explosion.png"),
        font: asset_server.load("fonts/press_start_2p/PressStart2P-Regular.ttf"),
        shoot_sfx: asset_server.load("sfx/shoot.wav"),
        explosion_sfx: asset_server.load("sfx/explosion.wav"),
        palette,
    });
    next_state.set(GameState::Title);
}
