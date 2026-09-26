use colored::Colorize;
use crate::args::{DBCommands, NSCommands};
use crate::core::{connect_cluster, list_namespaces, list_databases};
use crate::utils::{confirm, is_valid_identifier};

fn reject_if_invalid(name: &str) -> bool {
    if !is_valid_identifier(name) {
        eprintln!(
            "{} '{}' is not a valid identifier (letters, digits, underscore; must start with a letter).",
            "✖".red(),
            name
        );
        return true;
    }
    false
}

pub async fn ns_handler(command: NSCommands) {
    match command {
        NSCommands::New { name } => create_new_namespace(name).await,
        NSCommands::Remove { name } => remove_namespace(name).await,
    }
}

pub async fn db_handler(command: DBCommands) {
    match command {
        DBCommands::New { ns_name, db_name } => create_new_database(ns_name, db_name).await,
        DBCommands::Remove { ns_name, db_name } => remove_database(ns_name, db_name).await,
    }
}

async fn create_new_namespace(name: String) {
    if reject_if_invalid(&name) {
        return;
    }

    let database = connect_cluster().await;
    let namespaces = list_namespaces(&database).await;

    if namespaces.contains_key(&name) {
        println!("{} Namespace {} already exists!", "⚠".yellow(), name.cyan());
        return;
    }

    let query_string = format!("DEFINE NAMESPACE {};", name);
    match database.query(&query_string).await {
        Ok(_) => println!("{} Namespace {} created successfully!", "✔".green(), name.cyan()),
        Err(e) => eprintln!("{} Setup error: {}", "✖".red(), e),
    }
}

async fn remove_namespace(name: String) {
    if reject_if_invalid(&name) {
        return;
    }

    let database = connect_cluster().await;
    let namespaces = list_namespaces(&database).await;

    if !namespaces.contains_key(&name) {
        println!("{} Namespace {} does not exist!", "⚠".yellow(), name.cyan());
        return;
    }

    println!(
        "{} This will permanently delete namespace {} and EVERYTHING inside it (all databases, tables, and data).",
        "⚠".yellow().bold(),
        name.cyan().bold()
    );
    if !confirm(&format!("Type y to confirm deleting namespace '{}'", name)) {
        println!("{}", "Aborted, nothing was deleted.".bright_black());
        return;
    }

    let query_string = format!("REMOVE NAMESPACE {};", name);
    match database.query(&query_string).await {
        Ok(_) => println!("{} Namespace {} is removed!", "✔".green(), name.cyan()),
        Err(e) => eprintln!("{} Removal error: {}", "✖".red(), e),
    }
}

async fn create_new_database(ns_name: String, db_name: String) {
    if reject_if_invalid(&ns_name) || reject_if_invalid(&db_name) {
        return;
    }

    let database = connect_cluster().await;

    let namespaces = list_namespaces(&database).await;
    if !namespaces.contains_key(&ns_name) {
        eprintln!("{} Namespace {} does not exist!", "✖".red(), ns_name.cyan());
        return;
    }

    if let Err(e) = database.use_ns(&ns_name).await {
        eprintln!("{} Failed to bind namespace {}: {}", "✖".red(), ns_name.cyan(), e);
        return;
    }

    let databases = list_databases(&database).await;
    if databases.contains_key(&db_name) {
        println!("{} Database {} already exists inside namespace {}!", "⚠".yellow(), db_name.cyan(), ns_name.cyan());
        return;
    }

    let query_string = format!("DEFINE DATABASE {};", db_name);
    match database.query(&query_string).await {
        Ok(_) => println!("{} Database {} created successfully inside namespace {}!", "✔".green(), db_name.cyan(), ns_name.cyan()),
        Err(e) => eprintln!("{} Setup error: {}", "✖".red(), e),
    }
}

async fn remove_database(ns_name: String, db_name: String) {
    if reject_if_invalid(&ns_name) || reject_if_invalid(&db_name) {
        return;
    }

    let database = connect_cluster().await;

    let namespaces = list_namespaces(&database).await;
    if !namespaces.contains_key(&ns_name) {
        println!("{} Namespace {} does not exist, skipping database lookup!", "⚠".yellow(), ns_name.cyan());
        return;
    }

    if let Err(e) = database.use_ns(&ns_name).await {
        eprintln!("{} Failed to bind namespace {}: {}", "✖".red(), ns_name.cyan(), e);
        return;
    }

    let databases = list_databases(&database).await;
    if !databases.contains_key(&db_name) {
        println!("{} Database {} does not exist inside namespace {}!", "⚠".yellow(), db_name.cyan(), ns_name.cyan());
        return;
    }

    println!(
        "{} This will permanently delete database {} (all tables and data inside it) from namespace {}.",
        "⚠".yellow().bold(),
        db_name.cyan().bold(),
        ns_name.cyan()
    );
    if !confirm(&format!("Type y to confirm deleting database '{}'", db_name)) {
        println!("{}", "Aborted, nothing was deleted.".bright_black());
        return;
    }

    let query_string = format!("REMOVE DATABASE {};", db_name);
    match database.query(&query_string).await {
        Ok(_) => println!("{} Database {} is removed from {}!", "✔".green(), db_name.cyan(), ns_name.cyan()),
        Err(e) => eprintln!("{} Removal error: {}", "✖".red(), e),
    }
}