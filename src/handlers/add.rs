use colored::Colorize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use crate::constants::SudmConfig;
use crate::{error, success, warn};
use crate::utils::is_valid_identifier;

pub fn add_handler(name: String) {
    if !is_valid_identifier(&name) {
        error!("{} is not a valid identifier (letters, digits, underscore; must start with a letter).", name);
        return;
    }

    let filename = format!("{}.surql", name);

    let create_surql = |dir: &Path, kind: &str| {
        if let Err(e) = fs::create_dir_all(dir) {
            error!("Failed to create {} directory: {}", kind, e);
            return;
        }

        let path = dir.join(&filename);
        if path.exists() {
            warn!("{} file '{}' already exists!", kind, filename);
            return;
        }

        let template = format!("-- {}: {}\n-- Write your statements below.\n\n", kind.to_uppercase(), name);
        match File::create(&path).and_then(|mut f| f.write_all(template.as_bytes())) {
            Ok(_) => success!("Created {} file: {}", kind, path.display()),
            Err(e) => error!("Failed to create {} file: {}", kind, e),
        }
    };

    create_surql(&SudmConfig::current_schema_path(), "schema");
    create_surql(&SudmConfig::current_functions_path(), "function");
}