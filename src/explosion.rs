use crate::assets::GameAssets;
use crate::audio;
use crate::components::{EnemyDied, GameEntity, GameSpeed, GameState, PlayerDied};
use crate::random::{random_colour, random_float};
use bevy::prelude::*;
use bevy_rand::prelude::{GlobalEntropy, WyRand};

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_enemy_explosions,
                spawn_player_explosions,
                update_explosions,
                check_player_explosions,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Explosion {
    pub timer: f32,
}

#[derive(Component)]
pub struct PlayerExplosion;

#[derive(Resource)]
pub struct PlayerIsDead;

const EXPLOSION_LIFETIME: f32 = 0.375;

// spawns an explosion at the position of any enemy that has just died
fn spawn_enemy_explosions(
    mut commands: Commands,
    mut dead_events: EventReader<EnemyDied>,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
) {
    for EnemyDied(pos) in dead_events.read() {
        audio::play_with_volume(&mut commands, game_assets.explosion_sfx.clone(), 0.3);
        commands.spawn((
            Sprite {
                image: game_assets.explosion_texture.clone(),
                color: random_colour(&mut rng, &game_assets),
                ..Default::default()
            },
            Transform::from_translation(*pos),
            Explosion { timer: 0.0 },
            GameEntity,
        ));
    }
}

const NUM_PLAYER_EXPLOSIONS: i32 = 16;

// spawns multiple explosions at player's location
fn spawn_player_explosions(
    mut commands: Commands,
    mut player_died_events: EventReader<PlayerDied>,
    game_assets: Res<GameAssets>,
    mut rng: GlobalEntropy<WyRand>,
) {
    for PlayerDied(pos) in player_died_events.read() {
        info!("player died");
        audio::play_with_volume(&mut commands, game_assets.explosion_sfx.clone(), 0.5);
        for _ in 0..NUM_PLAYER_EXPLOSIONS {
            let offset_x = (random_float(&mut rng) - 0.5) * 20.0;
            let offset_y = (random_float(&mut rng) - 0.5) * 20.0;
            commands.spawn((
                Sprite {
                    image: game_assets.explosion_texture.clone(),
                    color: random_colour(&mut rng, &game_assets),
                    ..Default::default()
                },
                Transform::from_translation(*pos + Vec3::new(offset_x, offset_y, 0.)),
                Explosion {
                    timer: -2. * random_float(&mut rng), // stagger the explosion dissipation over time
                },
                PlayerExplosion,
                GameEntity,
            ));
        }
        commands.insert_resource(PlayerIsDead);
    }
}

// fades out explosions over time, despawning when done
fn update_explosions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Explosion, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut explosion, mut sprite) in query.iter_mut() {
        explosion.timer += time.delta_secs();
        if explosion.timer > EXPLOSION_LIFETIME {
            commands.entity(entity).despawn();
        } else {
            let alpha = if explosion.timer < EXPLOSION_LIFETIME / 2.0 {
                1.0
            } else {
                1.0 - (explosion.timer - EXPLOSION_LIFETIME / 2.0) / (EXPLOSION_LIFETIME / 2.0)
            };
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

// checks if the player is dead and player explosions have finished,
// in which case, return to title screen
fn check_player_explosions(
    mut commands: Commands,
    option_dead: Option<Res<PlayerIsDead>>,
    player_explosion_query: Query<Entity, With<PlayerExplosion>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_speed: ResMut<GameSpeed>,
) {
    if let Some(_) = option_dead {
        if player_explosion_query.is_empty() {
            next_state.set(GameState::Title);
            game_speed.value = 1.0;
            commands.remove_resource::<PlayerIsDead>();
            info!("player dead::switching to title");
        }
    }
}
