// border.rs
use crate::components::{GameEntity, GameState};
use crate::resolution::Resolution;
use crate::tilemap::{RENDERED_HEIGHT, RENDERED_WIDTH, TILE_SIZE};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
enum BorderSide {
    Left,
    Right,
    Top,
    Bottom,
}

pub struct BorderPlugin;

impl Plugin for BorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_borders) //, update_borders))
            .add_systems(
                Update,
                update_borders
                    .run_if(in_state(GameState::Playing).and(resource_changed::<Resolution>)),
            );
    }
}

fn spawn_borders(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::ZERO),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0001),
        BorderSide::Left,
        GameEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::ZERO),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0001),
        BorderSide::Right,
        GameEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::ZERO),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0001),
        BorderSide::Top,
        GameEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::ZERO),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0001),
        BorderSide::Bottom,
        GameEntity,
    ));
}

fn update_borders(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut borders: Query<(&BorderSide, &mut Transform, &mut Sprite)>,
) {
    let Ok(_window) = windows.single() else {
        return;
    };
    let Ok((camera, global_transform)) = cameras.single() else {
        return;
    };

    let ndc_to_world = |ndc: Vec3| -> Vec3 {
        camera
            .ndc_to_world(global_transform, ndc)
            .unwrap_or(Vec3::ZERO)
    };

    let world_left = ndc_to_world(Vec3::new(-1.0, 0.0, 0.0)).x;
    let world_right = ndc_to_world(Vec3::new(1.0, 0.0, 0.0)).x;
    let world_bottom = ndc_to_world(Vec3::new(0.0, -1.0, 0.0)).y;
    let world_top = ndc_to_world(Vec3::new(0.0, 1.0, 0.0)).y;

    let tilemap_half_w = (RENDERED_WIDTH as f32 / 2.0) * TILE_SIZE;
    let tilemap_half_h = (RENDERED_HEIGHT as f32 / 2.0) * TILE_SIZE;

    let tilemap_left = -tilemap_half_w;
    let tilemap_right = tilemap_half_w - TILE_SIZE;
    let tilemap_bottom = -tilemap_half_h;
    let tilemap_top = tilemap_half_h - TILE_SIZE;

    for (side, mut transform, mut sprite) in &mut borders {
        match side {
            BorderSide::Left => {
                let width = (tilemap_left - world_left).max(0.0);
                let height = world_top - world_bottom;
                let pos_x = world_left + width / 2.0;
                let pos_y = (world_top + world_bottom) / 2.0;
                transform.translation = Vec3::new(pos_x, pos_y, transform.translation.z);
                sprite.custom_size = Some(Vec2::new(width, height));
            }
            BorderSide::Right => {
                let width = (world_right - tilemap_right).max(0.0);
                let height = world_top - world_bottom;
                let pos_x = tilemap_right + width / 2.0;
                let pos_y = (world_top + world_bottom) / 2.0;
                transform.translation = Vec3::new(pos_x, pos_y, transform.translation.z);
                sprite.custom_size = Some(Vec2::new(width, height));
            }
            BorderSide::Top => {
                let height = (world_top - tilemap_top).max(0.0);
                let width = world_right - world_left;
                let pos_y = tilemap_top + height / 2.0;
                let pos_x = (world_right + world_left) / 2.0;
                transform.translation = Vec3::new(pos_x, pos_y, transform.translation.z);
                sprite.custom_size = Some(Vec2::new(width, height));
            }
            BorderSide::Bottom => {
                let height = (tilemap_bottom - world_bottom).max(0.0);
                let width = world_right - world_left;
                let pos_y = world_bottom + height / 2.0;
                let pos_x = (world_right + world_left) / 2.0;
                transform.translation = Vec3::new(pos_x, pos_y, transform.translation.z);
                sprite.custom_size = Some(Vec2::new(width, height));
            }
        }
    }
}
