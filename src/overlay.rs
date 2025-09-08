// overlay.rs
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState};
use crate::resolution::Resolution;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_overlay)
            .add_systems(OnExit(GameState::Playing), despawn_overlay)
            .add_systems(
                Update,
                update_overlay_size.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct Overlay;

fn spawn_overlay(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    resolution: Res<Resolution>,
) {
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            image: game_assets.overlay_texture.clone(),
            custom_size: Some(resolution.screen_dimensions), // Set initial size to window dimensions
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 999.0), // High z-index to render over tilemap but under UI
        Overlay,
        GameEntity,
    ));
}

fn despawn_overlay(mut commands: Commands, query: Query<Entity, With<Overlay>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_overlay_size(
    mut query: Query<&mut Sprite, With<Overlay>>,
    resolution: Res<Resolution>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if resolution.is_changed() {
        if let Ok(window) = window_query.single() {
            let window_size = Vec2::new(window.resolution.width(), window.resolution.height());
            for mut sprite in query.iter_mut() {
                sprite.custom_size = Some(3. * window_size);
            }
        }
    }
}
