use bevy::prelude::*;

use crate::assets;
use crate::audio;
use crate::collate_src;
use crate::components;
use crate::debug;
use crate::random;
use crate::resolution;
use crate::score;
use crate::tilemap;
use crate::title;
use crate::ui_scaling;
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            collate_src::CollateSrcPlugin,
            components::ComponentsPlugin,
            resolution::ResolutionPlugin,
            random::RandomPlugin,
            title::TitlePlugin,
            assets::AssetsPlugin,
            score::ScorePlugin,
            audio::AudioPlugin,
            debug::DebugPlugin,
            ui_scaling::UiScalingPlugin,
            tilemap::TilemapPlugin,
        ))
        .add_systems(Startup, setup_scene);
    }
}
fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
