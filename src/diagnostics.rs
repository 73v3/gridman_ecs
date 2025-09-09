// diagnostics.rs
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Adds frame time diagnostics (FPS, frame time, etc.)
            .add_plugins(FrameTimeDiagnosticsPlugin::default());
        // Logs diagnostics to the console at regular intervals
        //.add_plugins(LogDiagnosticsPlugin::default())
        // Optional diagnostic plugins (uncomment to enable)
        // .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
        // .add_plugins(bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<Texture>::default())
        // .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin::default());
        //
    }
}
