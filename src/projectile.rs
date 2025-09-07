// projectile.rs
use bevy::prelude::*;

//use crate::components::GameState;
//use crate::grid_movement::MovementSystems;

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct Bouncable {
    pub remaining: u32,
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, _app: &mut App) {
        // No additional setup needed, as AudioPlugin is included in DefaultPlugins
    }
}
