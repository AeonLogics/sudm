use colored::Colorize;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use crate::args::SchemaActions;
use crate::constants::SCHEMA_ROOT;
use crate::utils::is_valid_identifier;

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

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let migration_dir = schema_dir.join(format!("{timestamp}_{name}"));

    if let Err(e) = fs::create_dir_all(&migration_dir) {
        eprintln!("{} Failed to create migration folder: {}", "✖".red(), e);
        return;
    }

    let files: [(String, String); 4] = [
        (
            format!("{name}-schema.surql"),
            format!(
                "-- SCHEMA: {name} ({timestamp})\n\
                 -- Table/field/index definitions for THIS migration go here.\n\
                 -- Runs first, before {name}-functions.surql and {name}-events.surql.\n\n"
            ),
        ),
        (
            format!("{name}-functions.surql"),
            format!(
                "-- FUNCTIONS: {name} ({timestamp})\n\
                 -- Functions related to this migration go here. Runs after {name}-schema.surql.\n\
                 -- Use OVERWRITE so this stays safe to keep editing:\n\
                 -- DEFINE FUNCTION OVERWRITE fn::example($arg: string) {{ RETURN $arg; }};\n\n"
            ),
        ),
        (
            format!("{name}-events.surql"),
            format!(
                "-- EVENTS: {name} ({timestamp})\n\
                 -- Events (triggers on CREATE/UPDATE/DELETE) related to this migration go\n\
                 -- here. Runs last, after {name}-schema.surql and {name}-functions.surql.\n\
                 -- Use OVERWRITE so this stays safe to keep editing:\n\
                 -- DEFINE EVENT OVERWRITE example ON TABLE {name} WHEN $event = \"CREATE\" THEN {{ }};\n\n"
            ),
        ),
        (
            "rollback.surql".to_string(),
            format!(
                "-- ROLLBACK: {name} ({timestamp})\n\
                 -- Everything in {name}-schema.surql, {name}-functions.surql, AND\n\
                 -- {name}-events.surql is applied together as one unit. If you want to be\n\
                 -- able to fully revert this migration, write the complete rollback here --\n\
                 -- REMOVE TABLE / REMOVE FUNCTION / REMOVE EVENT for everything above.\n\
                 -- Leaving this blank means this migration cannot be undone; `sudm` will warn\n\
                 -- you and ask for confirmation before forgetting it was applied.\n\n"
            ),
        ),
    ];

    for (filename, template) in &files {
        let path = migration_dir.join(filename);
        if let Err(e) = fs::write(&path, template) {
            eprintln!("{} Failed to create {}: {}", "✖".red(), filename, e);
            return;
        }
    }

    println!("{} Created migration for '{}':", "✔".green(), name.cyan());
    for (filename, _) in &files {
        println!("    {}", migration_dir.join(filename).display());
    }
}