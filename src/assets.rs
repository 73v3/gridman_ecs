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
    pub font: Handle<Font>,
    pub shoot_sfx: Handle<AudioSource>,
    pub palette: Palette,
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let palette = Palette {
        // https://lospec.com/palette-list/gilt-8 by tomicit0
        colors: vec![
            Color::srgb(0.631, 0.224, 0.333),
            Color::srgb(0.761, 0.431, 0.522),
            Color::srgb(0.949, 0.729, 0.800),
            Color::srgb(1.000, 0.949, 0.918),
            Color::srgb(0.984, 0.906, 0.412),
            Color::srgb(0.894, 0.725, 0.169),
            Color::srgb(0.769, 0.416, 0.176),
            Color::srgb(0.506, 0.173, 0.137),
        ],
    };

    commands.insert_resource(GameAssets {
        wall_texture: asset_server.load("textures/wall.png"),
        player_texture: asset_server.load("textures/player.png"),
        reservation_texture: asset_server.load("textures/reservation.png"),
        font: asset_server.load("fonts/press_start_2p/PressStart2P-Regular.ttf"),
        shoot_sfx: asset_server.load("sfx/shoot.wav"),
        palette,
    });
    next_state.set(GameState::Title);
}
