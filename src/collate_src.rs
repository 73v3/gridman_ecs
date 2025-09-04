use bevy::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Plugin for collating all .rs files in src/ into a single assets/collated.txt file
#[derive(Debug, Clone, Copy, Default)]
pub struct CollateSrcPlugin;

impl Plugin for CollateSrcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, collate_source_files);
    }
}

/// System to read .rs files from src/ and write them to assets/collated.txt
fn collate_source_files() {
    // Ensure the assets directory exists
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir).expect("Failed to create assets directory");
    }

    // Open the output file
    let mut output_file =
        File::create("assets/collated_src.txt").expect("Failed to create collated.txt");

    // Read all files in src/
    let src_dir = Path::new("src");
    if src_dir.is_dir() {
        for entry in fs::read_dir(src_dir).expect("Failed to read src directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            // Process only .rs files, excluding collate_src.rs
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if file_name != "collate_src.rs" {
                        // Read file contents
                        let contents = fs::read_to_string(&path)
                            .expect(&format!("Failed to read file: {}", file_name));

                        // Write tagged contents to output file
                        writeln!(output_file, "<{}>", file_name)
                            .expect("Failed to write file name tag");
                        write!(output_file, "{}", contents).expect("Failed to write file contents");
                        writeln!(output_file, "</{}>\n", file_name)
                            .expect("Failed to write closing tag");
                    }
                }
            }
        }
    }
}
