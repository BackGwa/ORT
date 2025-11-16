use ort::parse_ort;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ort2json <file.ort> [-o <output_dir>]");
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

    // Parse ORT to OrtValue
    let ort_value = match parse_ort(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // Convert OrtValue to JSON Value
    let json_value: serde_json::Value = ort_value.into();

    // Convert to JSON string
    let json_string = match serde_json::to_string_pretty(&json_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize JSON: {}", e);
            process::exit(1);
        }
    };

    // Determine output path
    let input_path_obj = Path::new(input_path);
    let output_path = if let Some(dir) = output_dir {
        let file_name = input_path_obj
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        dir.join(format!("{}.json", file_name))
    } else {
        input_path_obj.with_extension("json")
    };

    // Write output file
    if let Err(e) = fs::write(&output_path, json_string) {
        eprintln!("Failed to write file '{}': {}", output_path.display(), e);
        process::exit(1);
    }
}
