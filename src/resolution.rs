use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};

pub struct ResolutionPlugin;

impl Plugin for ResolutionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_resolution).add_systems(
            Update,
            (handle_window_resize, update_camera_projection).chain(),
        );
    }
}

// Increasing this value will result in the projection zooming out, showing more of the render area
const MASTER_SCALE: f32 = 4.0;

#[derive(Resource)]
pub struct Resolution {
    // Pixel dimensions of the screen (width, height)
    pub screen_dimensions: Vec2,
    // The ratio of a pixel in our sprites to one on screen
    pub pixel_ratio: f32,
    // Base resolution for scaling (e.g., the design resolution)
    pub base_resolution: Vec2,
    // Decrease to show more onscreen 0..1
    pub zoom: f32,
}

fn setup_resolution(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    if let Ok(window) = window_query.single() {
        let width = window.resolution.width();
        let height = window.resolution.height();

        commands.insert_resource(Resolution {
            screen_dimensions: Vec2::new(width, height),
            pixel_ratio: window.scale_factor() as f32,
            base_resolution: Vec2::new(800.0, 600.0),
            zoom: 1.0,
        });
    } else {
        error!("No primary window found during resolution setup");
        // Fallback to default resolution
        commands.insert_resource(Resolution {
            screen_dimensions: Vec2::new(800.0, 600.0),
            pixel_ratio: 1.0,
            base_resolution: Vec2::new(800.0, 600.0),
            zoom: 1.0,
        });
    }
}

fn handle_window_resize(
    mut resize_events: EventReader<WindowResized>,
    mut resolution: ResMut<Resolution>,
    // Query for the Entity and the Window component of the primary window
    window_query: Query<(Entity, &Window), With<PrimaryWindow>>,
) {
    // Get the entity and component for the primary window
    if let Ok((primary_window_entity, primary_window)) = window_query.single() {
        for event in resize_events.read() {
            // Compare the event's entity with the primary window's entity
            if event.window == primary_window_entity {
                resolution.screen_dimensions = Vec2::new(event.width, event.height);
                resolution.pixel_ratio = primary_window.scale_factor() as f32;
                info!("Window resized to {}x{}", event.width, event.height);
            }
        }
    }
}

fn update_camera_projection(
    resolution: Res<Resolution>,
    mut query: Query<&mut Projection, With<Camera2d>>,
) {
    if resolution.is_changed() {
        for mut projection in query.iter_mut() {
            if let Projection::Orthographic(ref mut ortho) = &mut *projection {
                let scale_x = resolution.screen_dimensions.x / resolution.base_resolution.x;
                let scale_y = resolution.screen_dimensions.y / resolution.base_resolution.y;
                // Use the smaller scale to maintain aspect ratio and avoid stretching
                let scale = scale_x.min(scale_y) * resolution.pixel_ratio;

                ortho.scale = (MASTER_SCALE * resolution.zoom) * 1.0 / scale;
                info!("Updated camera projection scale: {}", ortho.scale);
            }
        }
    }
}
