// collider.rs
use crate::components::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub struct Collider {
    pub size: Vec2,
}

#[derive(Event)]
pub struct Collision {
    pub a: Entity,
    pub b: Entity,
}

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, _app: &mut App) {
        /*
        app.add_event::<Collision>().add_systems(
            Update,
            detect_collisions.run_if(in_state(GameState::Playing)),
        );
        */
    }
}

fn _detect_collisions(
    mut events: EventWriter<Collision>,
    query: Query<(Entity, &Transform, &Collider)>,
) {
    let entities: Vec<(Entity, Vec2, Vec2)> = query
        .iter()
        .map(|(e, t, c)| (e, t.translation.xy(), c.size))
        .collect();

    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (a, pos_a, size_a) = entities[i];
            let (b, pos_b, size_b) = entities[j];

            if _aabb_overlap(pos_a, size_a, pos_b, size_b) {
                events.write(Collision { a, b });
            }
        }
    }
}

fn _aabb_overlap(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let half1 = size1 / 2.0;
    let half2 = size2 / 2.0;
    let min1 = pos1 - half1;
    let max1 = pos1 + half1;
    let min2 = pos2 - half2;
    let max2 = pos2 + half2;

    min1.x < max2.x && max1.x > min2.x && min1.y < max2.y && max1.y > min2.y
}
