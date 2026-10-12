use colored::Colorize;
use dotenv::var;
use serde::Deserialize;
use std::collections::BTreeMap;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;
use crate::success;

#[derive(Deserialize, Debug, SurrealValue)]
struct RootInfo {
    namespaces: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct NamespaceInfo {
    databases: BTreeMap<String, String>,
}

fn require_var(key: &str) -> String {
    let val = var(key).unwrap_or_else(|_| panic!("{key} must be set"));
    if val.trim().is_empty() {
        panic!("{key} must not be empty");
    }
    val
}

pub async fn connection() -> Surreal<Any> {
    let db = connect_cluster().await;
    let ns = require_var("NAMESPACE");
    let db_name = require_var("DATABASE_NAME");

    let namespaces = list_namespaces(&db).await;
    if !namespaces.contains_key(&ns) {
        success!("{} Namespace '{}' not found. Creating it...", "ℹ".yellow(), ns);
        if let Err(e) = db.query(format!("DEFINE NAMESPACE `{}`;", ns)).await {
            panic!("Failed to auto-create namespace '{}': {}", ns, e);
        }
    }
    db.use_ns(&ns).await.expect("Failed to bind namespace");

    let databases = list_databases(&db).await;
    if !databases.contains_key(&db_name) {
        println!("{} Database '{}' not found in namespace '{}'. Creating it...", "ℹ".yellow(), db_name, ns);
        if let Err(e) = db.query(format!("DEFINE DATABASE `{}`;", db_name)).await {
            panic!("Failed to auto-create database '{}': {}", db_name, e);
        }
    }

    db.use_db(&db_name).await.expect("Failed to bind database");
    success!("Bound to namespace '{}', database '{}'. \n", ns, db_name);

    db
}

pub async fn connect_cluster() -> Surreal<Any> {
    let mut url = require_var("DATABASE_URL");
    let user = require_var("SURREAL_USER");
    let pass = require_var("SURREAL_PASS");

    if !url.contains("://") {
        url = format!("ws://{}", url);
    }

    let database = connect(&url).await.unwrap_or_else(|err| {
        panic!(
            "Failed to connect to SurrealDB endpoint at '{}'. \nUnderlying driver error: {}",
            url, err
        )
    });

    database
        .signin(Root { username: user, password: pass })
        .await
        .expect("Database authentication failed with the provided credentials");

    println!("{}", "Connected and authenticated to SurrealDB cluster.".green());
    database
}

pub async fn list_namespaces(db: &Surreal<Any>) -> BTreeMap<String, String> {
    let mut res = match db.query("INFO FOR ROOT;").await {
        Ok(r) => r,
        Err(_) => return BTreeMap::new(),
    };
    res.take::<Option<RootInfo>>(0)
        .ok()
        .flatten()
        .map(|info| info.namespaces)
        .unwrap_or_default()
}

pub async fn list_databases(db: &Surreal<Any>) -> BTreeMap<String, String> {
    let mut res = match db.query("INFO FOR NS;").await {
        Ok(r) => r,
        Err(_) => return BTreeMap::new(),
    };
    res.take::<Option<NamespaceInfo>>(0)
        .ok()
        .flatten()
        .map(|info| info.databases)
        .unwrap_or_default()
}