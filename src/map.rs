use crate::components::GameState;
use bevy::prelude::*;

#[derive(Resource)]
struct MapHandle(Handle<Image>);

#[derive(Resource)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
    pub is_wall: Vec<bool>,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), load_map)
            .add_systems(Update, process_map.run_if(resource_exists::<MapHandle>));
    }
}

fn load_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<Image> = asset_server.load("maps/0.png");
    commands.insert_resource(MapHandle(handle));
}

fn process_map(mut commands: Commands, map_handle: Res<MapHandle>, images: Res<Assets<Image>>) {
    if let Some(image) = images.get(&map_handle.0) {
        let width = image.size().x as u32;
        let height = image.size().y as u32;
        let mut is_wall = vec![false; (width * height) as usize];

        if let Some(data) = &image.data {
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    // Ensure we don't go out of bounds
                    if idx + 2 < data.len() {
                        let r = data[idx] as f32 / 255.0;
                        let g = data[idx + 1] as f32 / 255.0;
                        let b = data[idx + 2] as f32 / 255.0;
                        is_wall[(y * width + x) as usize] = r > 0.0 || g > 0.0 || b > 0.0;
                    }
                }
            }
        }

        commands.insert_resource(MapData {
            width,
            height,
            is_wall,
        });
        commands.remove_resource::<MapHandle>();
    }
}
