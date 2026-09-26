use colored::Colorize;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use crate::args::SchemaActions;
use crate::utils::is_valid_identifier;
use crate::constants::SCHEMA_ROOT;


pub async fn schema_handler(actions: SchemaActions) {
    match actions {
        SchemaActions::New { name } => create_new_schema(name),
        SchemaActions::Add { name } => add_to_schema(name),
    }
}

fn schema_dir(name: &str) -> PathBuf {
    PathBuf::from(SCHEMA_ROOT).join(name)
}

pub fn create_new_schema(name: String) {
    if !is_valid_identifier(&name) {
        eprintln!(
            "{} '{}' is not a valid schema name (letters, digits, underscore; must start with a letter).",
            "✖".red(),
            name
        );
        return;
    }

    let dir = schema_dir(&name);

    if dir.exists() {
        println!("{} Schema '{}' already exists at {}", "⚠".yellow(), name.cyan(), dir.display());
        return;
    }

    match fs::create_dir_all(&dir) {
        Ok(_) => println!("{} Created schema '{}' at {}", "✔".green(), name.cyan(), dir.display()),
        Err(e) => eprintln!("{} Failed to create schema folder: {}", "✖".red(), e),
    }
}

pub fn add_to_schema(name: String) {
    if !is_valid_identifier(&name) {
        eprintln!(
            "{} '{}' is not a valid schema name (letters, digits, underscore; must start with a letter).",
            "✖".red(),
            name
        );
        return;
    }

    let schema_dir = schema_dir(&name);

    if !schema_dir.exists() {
        eprintln!(
            "{} Schema '{}' does not exist yet. Run `schema new {}` first.",
            "✖".red(),
            name.cyan(),
            name
        );
        return;
    }

    // e.g. migrations/schema/user/20260925095558_user/
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let migration_dir = schema_dir.join(format!("{timestamp}_{name}"));

    if let Err(e) = fs::create_dir_all(&migration_dir) {
        eprintln!("{} Failed to create migration folder: {}", "✖".red(), e);
        return;
    }

    let up_path = migration_dir.join("up.surql");
    let down_path = migration_dir.join("down.surql");

    let up_template = format!("-- Migration UP: {name} ({timestamp})\n\n");
    let down_template = format!("-- Migration DOWN: {name} ({timestamp})\n\n");

    if let Err(e) = fs::write(&up_path, up_template) {
        eprintln!("{} Failed to create up.surql: {}", "✖".red(), e);
        return;
    }

    if let Err(e) = fs::write(&down_path, down_template) {
        eprintln!("{} Failed to create down.surql: {}", "✖".red(), e);
        return;
    }

    println!("{} Created migration for '{}':", "✔".green(), name.cyan());
    println!("    {}", up_path.display());
    println!("    {}", down_path.display());
}