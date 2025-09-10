// src/collate_src.rs
use bevy::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Plugin for collating all .rs files in src/ into a single assets/collated_src.txt file
#[derive(Debug, Clone, Copy, Default)]
pub struct CollateSrcPlugin;

impl Plugin for CollateSrcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomPrompt>()
            .add_systems(PreStartup, collate_source_files);
    }
}

/// Resource to hold the custom instruction prompt
#[derive(Resource)]
pub struct CustomPrompt(String);

impl Default for CustomPrompt {
    fn default() -> Self {
        CustomPrompt(
            r#"
As usual, we require an ECS-first, performant, and elegant solution that perfectly solves our problem while adhering to Bevy's best practices for modularity, efficiency, and maintainability.

We are using Bevy 0.16.1, so avoid all deprecated APIs or fields (e.g., use .delta_secs() instead of .delta_seconds()). Refer to the existing codebase for examples of correct syntax and patterns when in doubt, and ensure compatibility with Bevy's current features.

Before writing any code, think step-by-step through your design strategy out loud: outline the problem requirements, brainstorm multiple architectural options (e.g., different component/system/query designs), evaluate their trade-offs in terms of performance, simplicity, and extensibility, and justify your selection of the optimal approach.

If applicable, use the code_execution tool to prototype or verify non-Bevy-specific logic (e.g., file I/O, string manipulation) during your reasoning phase. Focus on edge cases or complex logic that benefits from isolated testing, but avoid attempting to execute Bevy-specific ECS or rendering functionality.

Professionally comment all new code and retain all existing comments.

Output all changed or new files in full, each in its own dedicated codebox. Do not output unchanged files or partial diffs—provide complete, compilable files.

After outputting all files, provide a concise summary (outside of codeboxes) that recaps the key changes, evaluates their merits (e.g., how they improve performance, readability, or solve edge cases), and suggests any potential future improvements.
            "#.trim().to_string(),
        )
    }
}

/// System to read .rs files from src/ and write them to assets/collated_src.txt
fn collate_source_files(prompt: Res<CustomPrompt>) {
    // Ensure the assets directory exists
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir).expect("Failed to create assets directory");
    }

    // Open the output file
    let mut output_file =
        File::create("assets/collated_src.txt").expect("Failed to create collated_src.txt");

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

    // Append the custom prompt
    writeln!(output_file, "<task rules>").expect("Failed to write prompt opening tag");
    write!(output_file, "{}", prompt.0).expect("Failed to write prompt contents");
    writeln!(output_file, "\n</task rules>").expect("Failed to write prompt closing tag");
    writeln!(output_file, "<task>").expect("Failed to write prompt opening task tag");
    writeln!(output_file, " ").expect("Failed to write instructions tag");
    writeln!(output_file, "</task>").expect("Failed to write closing task tag");
}
