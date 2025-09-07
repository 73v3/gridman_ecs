// grid_movement.rs
use bevy::ecs::schedule::SystemSet;
use bevy::prelude::*;

use crate::components::GameState;
use crate::map::MapData;
use crate::projectile::{Bouncable, Projectile};
//use crate::resolution::Resolution;

use crate::tilemap::{MapOffset, TileOffset, HALF_HEIGHT, HALF_WIDTH, TILE_SIZE};

#[derive(Component)]
pub struct GridMover {
    pub grid_pos: IVec2,
    pub direction: IVec2,
    pub progress: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct IntendedDirection(pub IVec2);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MovementSystems {
    Input,
    UpdateMover,
    UpdatePosition,
    AdjustScroll,
}

pub struct GridMovementPlugin;

impl Plugin for GridMovementPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                MovementSystems::Input,
                MovementSystems::UpdateMover.after(MovementSystems::Input),
                MovementSystems::UpdatePosition.after(MovementSystems::UpdateMover),
                MovementSystems::AdjustScroll.after(MovementSystems::UpdatePosition),
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            update_grid_movement.in_set(MovementSystems::UpdateMover),
        )
        .add_systems(
            Update,
            update_grid_positions.in_set(MovementSystems::UpdatePosition),
        );
    }
}

fn update_grid_movement(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut GridMover,
        &mut IntendedDirection,
        Option<&mut Bouncable>,
        Option<&Projectile>,
    )>,
    time: Res<Time>,
    map_data: Res<MapData>,
) {
    for (entity, mut mover, mut intended, bouncable, projectile) in &mut query {
        if mover.direction == IVec2::ZERO {
            let new_dir = intended.0;
            if new_dir != IVec2::ZERO {
                let next_tile = mover.grid_pos + new_dir;
                if !is_wall(next_tile, &map_data) {
                    mover.direction = new_dir;
                    mover.progress = 0.0;
                }
            }
        } else {
            let dir_vec = mover.direction.as_vec2();
            let dist_factor = dir_vec.length();
            if dist_factor == 0.0 {
                continue;
            }
            let inc = mover.speed * time.delta_secs() / (TILE_SIZE * dist_factor);
            mover.progress += inc;

            if mover.progress >= 1.0 {
                let current_direction = mover.direction;
                mover.grid_pos += current_direction;
                let is_continuing =
                    intended.0 == current_direction && current_direction != IVec2::ZERO;
                if is_continuing {
                    let next_tile = mover.grid_pos + current_direction;
                    if !is_wall(next_tile, &map_data) {
                        mover.progress -= 1.0;
                    } else {
                        // Wall detected ahead
                        let can_bounce = bouncable.as_ref().map_or(false, |b| b.remaining > 0);
                        if can_bounce {
                            let new_dir =
                                calculate_reflection(current_direction, mover.grid_pos, &map_data);
                            mover.direction = new_dir;
                            intended.0 = new_dir;
                            if let Some(mut b) = bouncable {
                                b.remaining -= 1;
                            }
                            let old_length = current_direction.as_vec2().length();
                            let new_length = new_dir.as_vec2().length();
                            mover.progress -= 1.0;
                            if new_length > 0.0 && old_length > 0.0 {
                                mover.progress *= old_length / new_length;
                            }
                        } else {
                            mover.progress = 0.0;
                            mover.direction = IVec2::ZERO;
                            intended.0 = IVec2::ZERO;
                            if projectile.is_some() {
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                } else {
                    mover.progress = 0.0;
                    let new_dir = intended.0;
                    if new_dir != IVec2::ZERO {
                        let next_tile = mover.grid_pos + new_dir;
                        if !is_wall(next_tile, &map_data) {
                            mover.direction = new_dir;
                        } else {
                            mover.direction = IVec2::ZERO;
                        }
                    } else {
                        mover.direction = IVec2::ZERO;
                    }
                }
            }
        }
    }
}

fn calculate_reflection(dir: IVec2, grid_pos: IVec2, map_data: &MapData) -> IVec2 {
    let dx = dir.x;
    let dy = dir.y;
    let horiz_next = grid_pos + IVec2::new(dx, 0);
    let vert_next = grid_pos + IVec2::new(0, dy);
    let horiz_clear = !is_wall(horiz_next, map_data);
    let vert_clear = !is_wall(vert_next, map_data);
    if horiz_clear {
        IVec2::new(dx, -dy)
    } else if vert_clear {
        IVec2::new(-dx, dy)
    } else {
        IVec2::new(-dx, -dy)
    }
}

fn update_grid_positions(
    map_offset: Res<MapOffset>,
    tile_offset: Res<TileOffset>,
    mut query: Query<(&GridMover, &mut Transform)>,
) {
    for (mover, mut trans) in &mut query {
        let effective_pos = mover.grid_pos.as_vec2() + mover.direction.as_vec2() * mover.progress;
        let x =
            (effective_pos.x - map_offset.0.x as f32 - HALF_WIDTH) * TILE_SIZE + tile_offset.0.x;
        let y =
            (effective_pos.y - map_offset.0.y as f32 - HALF_HEIGHT) * TILE_SIZE + tile_offset.0.y;
        trans.translation.x = x;
        trans.translation.y = y;
    }
}

pub fn is_wall(pos: IVec2, map: &MapData) -> bool {
    if pos.x < 0 || pos.y < 0 || pos.x >= map.width as i32 || pos.y >= map.height as i32 {
        return true;
    }
    let x = pos.x as u32;
    let y = pos.y as u32;
    let flipped_y = map.height - 1 - y;
    let idx = (flipped_y * map.width + x) as usize;
    map.is_wall.get(idx).copied().unwrap_or(true)
}
