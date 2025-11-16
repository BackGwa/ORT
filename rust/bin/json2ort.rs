use ort_rs::{generate_ort, OrtValue};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: json2ort <file.json> [-o <output_dir>]");
        process::exit(1);
    }

    let input_path = &args[1];
    let output_dir = if args.len() >= 4 && args[2] == "-o" {
        Some(PathBuf::from(&args[3]))
    } else {
        None
    };

    // Read input file
    let content = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    // Parse JSON
    let json_value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            process::exit(1);
        }
    };

    // Convert JSON Value to OrtValue
    let ort_value: OrtValue = json_value.into();

    // Generate ORT
    let ort_string = generate_ort(&ort_value);

    // Determine output path
    let input_path_obj = Path::new(input_path);
    let output_path = if let Some(dir) = output_dir {
        let file_name = input_path_obj
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        dir.join(format!("{}.ort", file_name))
    } else {
        input_path_obj.with_extension("ort")
    };

    // Write output file
    if let Err(e) = fs::write(&output_path, ort_string) {
        eprintln!("Failed to write file '{}': {}", output_path.display(), e);
        process::exit(1);
    }
}
