// score.rs
use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{GameEntity, GameState};

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScoreChanged>()
            .add_systems(OnEnter(GameState::Playing), setup_score)
            .add_systems(
                Update,
                (update_score, update_score_display)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
pub struct Score {
    pub value: u32,
}

#[derive(Event)]
pub struct ScoreChanged;

#[derive(Component)]
struct ScoreText;

fn setup_score(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.insert_resource(Score { value: 0 });

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
            Text::new("0000".to_string()),
            TextFont {
                font: game_assets.font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(game_assets.palette.colors[3]),
            TextLayout::new_with_justify(JustifyText::Center),
            ScoreText,
        ));
    });
}

fn update_score(mut score: ResMut<Score>, mut events: EventReader<ScoreChanged>) {
    for _ in events.read() {
        score.value += 1;
        if score.value > 9999 {
            score.value = 9999;
        }
    }
}

fn update_score_display(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            text.0 = format!("{:04}", score.value);
        }
    }
}
