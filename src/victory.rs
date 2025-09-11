// victory.rs
use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{EnemyGroupSize, GameEntity, GameState};
use crate::enemy::Enemy;
use crate::player::Player;

pub struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Victory), spawn_victory)
            .add_systems(OnExit(GameState::Victory), (despawn_victory, cleanup_game))
            .add_systems(
                Update,
                (
                    check_for_victory.run_if(in_state(GameState::Playing)),
                    handle_victory_timer.run_if(in_state(GameState::Victory)),
                ),
            );
    }
}

#[derive(Resource)]
struct VictoryTimer(Timer);

#[derive(Component)]
struct VictoryText;

fn spawn_victory(mut commands: Commands, game_assets: Res<GameAssets>) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            VictoryText,
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Text::new("VICTORY"),
            TextFont {
                font: game_assets.font.clone(),
                font_size: 40.0,
                ..default()
            },
            TextColor(game_assets.palette.colors[12]),
            TextLayout::new_with_justify(JustifyText::Center),
        ));
    });

    // Insert the timer resource
    commands.insert_resource(VictoryTimer(Timer::from_seconds(5.0, TimerMode::Once)));
}

fn despawn_victory(mut commands: Commands, query: Query<Entity, With<VictoryText>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_victory_timer(
    mut timer: ResMut<VictoryTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut enemy_group_size: ResMut<EnemyGroupSize>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        const MAX_PER_TYPE: u32 = 2048;
        enemy_group_size.0 = (enemy_group_size.0 * 2).min(MAX_PER_TYPE);
        next_state.set(GameState::Playing);
    }
}

fn check_for_victory(
    enemy_query: Query<(), With<Enemy>>,
    player_query: Query<(), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if enemy_query.is_empty() && !player_query.is_empty() {
        next_state.set(GameState::Victory);
    }
}

fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    info!("Cleaning up game entities");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
