use colored::Colorize;
use crate::args::{DBCommands, NSCommands};
use crate::core::connection;
use serde::Deserialize;
use std::collections::BTreeMap;
use surrealdb::types::SurrealValue;

#[derive(SurrealValue, Deserialize, Debug)]
struct RootInfo {
    namespaces: BTreeMap<String, String>,
}

#[derive(SurrealValue, Deserialize, Debug)]
struct NamespaceInfo {
    databases: BTreeMap<String, String>,
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
    let database = connection().await;

    let mut root_response = match database.query("INFO FOR ROOT;").await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} Failed to check cluster state: {}", "✖".red(), e);
            return;
        }
    };

    let namespaces = root_response
        .take::<Option<RootInfo>>(0)
        .unwrap()
        .map(|info| info.namespaces)
        .unwrap_or_default();

    if namespaces.contains_key(&name) {
        println!("{} Namespace {} already exists!", "⚠".yellow(), name.cyan());
        return;
    }

    let query_string = format!("DEFINE NAMESPACE {};", name);
    match database.query(&query_string).await {
        Ok(_) => {
            println!("{} Namespace {} created successfully!", "✔".green(), name.cyan());
        }
        Err(e) => {
            eprintln!("{} Setup error: {}", "✖".red(), e);
        }
    }
}

async fn remove_namespace(name: String) {
    let database = connection().await;

    let mut root_response = match database.query("INFO FOR ROOT;").await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} Failed to check cluster state: {}", "✖".red(), e);
            return;
        }
    };

    let namespaces = root_response
        .take::<Option<RootInfo>>(0)
        .unwrap()
        .map(|info| info.namespaces)
        .unwrap_or_default();

    if !namespaces.contains_key(&name) {
        println!("{} Namespace {} does not exist!", "⚠".yellow(), name.cyan());
        return;
    }

    let _ = database.use_ns("").use_db("").await;
    let query_string = format!("REMOVE NAMESPACE {};", name);

    match database.query(&query_string).await {
        Ok(_) => {
            println!("{} Namespace {} is removed!", "✔".green(), name.cyan());
        }
        Err(e) => {
            eprintln!("{} Removal error: {}", "✖".red(), e);
        }
    }
}

async fn create_new_database(ns_name: String, db_name: String) {
    let database = connection().await;

    if let Err(e) = database.use_ns(&ns_name).await {
        eprintln!("{} Namespace {} does not exist or is locked: {}", "✖".red(), ns_name.cyan(), e);
        return;
    }

    let mut ns_response = match database.query("INFO FOR NS;").await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} Failed to verify database layout: {}", "✖".red(), e);
            return;
        }
    };

    let databases = ns_response
        .take::<Option<NamespaceInfo>>(0)
        .unwrap()
        .map(|info| info.databases)
        .unwrap_or_default();

    if databases.contains_key(&db_name) {
        println!("{} Database {} already exists inside namespace {}!", "⚠".yellow(), db_name.cyan(), ns_name.cyan());
        return;
    }

    let query_string = format!("DEFINE DATABASE {};", db_name);
    match database.query(&query_string).await {
        Ok(_) => {
            println!("{} Database {} created successfully inside namespace {}!", "✔".green(), db_name.cyan(), ns_name.cyan());
        }
        Err(e) => {
            eprintln!("{} Setup error: {}", "✖".red(), e);
        }
    }
}

async fn remove_database(ns_name: String, db_name: String) {
    let database = connection().await;

    if let Err(_) = database.use_ns(&ns_name).await {
        println!("{} Namespace {} does not exist, skipping database lookup!", "⚠".yellow(), ns_name.cyan());
        return;
    }

    let mut ns_response = match database.query("INFO FOR NS;").await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} Failed to verify database layout: {}", "✖".red(), e);
            return;
        }
    };

    let databases = ns_response
        .take::<Option<NamespaceInfo>>(0)
        .unwrap()
        .map(|info| info.databases)
        .unwrap_or_default();

    if !databases.contains_key(&db_name) {
        println!("{} Database {} does not exist inside namespace {}!", "⚠".yellow(), db_name.cyan(), ns_name.cyan());
        return;
    }

    let query_string = format!("REMOVE DATABASE {};", db_name);
    match database.query(&query_string).await {
        Ok(_) => {
            println!("{} Database {} is removed from {}!", "✔".green(), db_name.cyan(), ns_name.cyan());
        }
        Err(e) => {
            eprintln!("{} Removal error: {}", "✖".red(), e);
        }
    }
}
