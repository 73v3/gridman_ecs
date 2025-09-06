use bevy::prelude::*;

//link our modules to our project

pub mod assets;
pub mod audio;
pub mod collate_src;
pub mod components;
pub mod custom_window;
pub mod debug;
pub mod game;
pub mod map;
pub mod player;
pub mod random;
pub mod resolution;
pub mod score;
pub mod tilemap;
pub mod title;
pub mod ui_scaling;

fn main() {
    App::new()
        .add_plugins((custom_window::CustomWindowPlugin, game::GamePlugin))
        .run();
}
