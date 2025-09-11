// score.rs
use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{EnemyDied, GameEntity, GameState};
use crate::enemy::{spawn_enemies, Enemy}; // Added spawn_enemies import

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            setup_enemy_count.after(spawn_enemies), // Ensure runs after enemies are spawned
        )
        .add_systems(
            Update,
            (update_enemy_count, update_enemy_count_display)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource)]
pub struct EnemyCount {
    pub value: u32,
}

#[derive(Component)]
struct EnemyCountText;

fn setup_enemy_count(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    enemy_query: Query<(), With<Enemy>>,
) {
    // Count the number of enemies at the start of the game
    let initial_count = enemy_query.iter().len() as u32;
    commands.insert_resource(EnemyCount {
        value: initial_count,
    });

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GameEntity,
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Text::new(format!("remaining: {}", initial_count)),
            TextFont {
                font: game_assets.font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(game_assets.palette.colors[3]),
            TextLayout::new_with_justify(JustifyText::Center),
            EnemyCountText,
        ));
    });
}

fn update_enemy_count(mut enemy_count: ResMut<EnemyCount>, mut events: EventReader<EnemyDied>) {
    for _ in events.read() {
        if enemy_count.value > 0 {
            enemy_count.value -= 1;
        }
    }
}

fn update_enemy_count_display(
    enemy_count: Res<EnemyCount>,
    mut query: Query<&mut Text, With<EnemyCountText>>,
) {
    if enemy_count.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            text.0 = format!("remaining: {}", enemy_count.value);
        }
    }
}
