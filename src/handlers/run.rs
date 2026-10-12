use std::fs::{create_dir_all, read_dir, read_to_string};
use crate::constants::SudmConfig;
use crate::{error, success, warn};
use colored::Colorize;
use crate::actions::RunActions;
use crate::core::connection;

pub async fn run_handler(actions: RunActions) {
    match actions {
        RunActions::Setup => run_setup(),
        RunActions::Migration => run_mig().await,
    }
}

fn run_setup() {
    let flux_paths = SudmConfig::flux_schema_paths();

    println!("Initiating surql_flux \n");

    flux_paths.iter().for_each(|path| {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if path.exists() {
            warn!("Flux {} already exists", file_name);
        } else {
            match create_dir_all(path) {
                Ok(_) => {
                    success!("Created Flux {}", file_name);
                },
                Err(e) => {
                    error!("Error Creating Flux {}, Error: {}", file_name, e);
                }
            }
        }
    });
}

async fn run_mig() {
    let db = connection().await;

    // Collect schema, function, and event paths
    let dirs = vec![
        SudmConfig::current_schema_path(),
        SudmConfig::current_functions_path(),
        SudmConfig::current_events_path(), // Add your events folder here
    ];

    let mut entries = Vec::new();

    for dir in dirs {
        if let Ok(dir_iter) = read_dir(&dir) {
            let files = dir_iter
                .filter_map(|res| res.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("surql"));
            entries.extend(files);
        }
    }

    entries.sort();

    if entries.is_empty() {
        warn!("No migration files found!");
        return;
    }

    for path in entries {
        let filename = path.file_name().unwrap().to_string_lossy();
        let query_content = match read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read file {}: {}", filename, e);
                return;
            }
        };

        match db.query(&query_content).await {
            Ok(_) => success!("Applied: {}", filename),
            Err(e) => {
                error!("Failed at {}: {}", filename, e);
                return;
            }
        }
    }

    println!("\n");
    success!("All schemas, functions, and events applied successfully!");
}