use colored::Colorize;
use dotenv::var;
use serde::Deserialize;
use std::collections::BTreeMap;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;

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

pub async fn connect_cluster() -> Surreal<Any> {
    let url = require_var("DATABASE_URL");
    let user = require_var("SURREAL_USER");
    let pass = require_var("SURREAL_PASS");

    if !url.starts_with("ws://")
        && !url.starts_with("wss://")
        && !url.starts_with("http://")
        && !url.starts_with("https://")
    {
        panic!(
            "\n[Configuration Error]: Invalid DATABASE_URL ('{}').\n\
            SurrealDB requires an explicit protocol engine prefix.\n\
            Please update your environment file to include a scheme like:\n\
            - ws://localhost:8000\n\
            - wss://cloud.surrealdb.com\n\
            - http://localhost:8000\n",
            url
        );
    }

    let database = connect(&url).await.unwrap_or_else(|err| {
        panic!(
            "Failed to connect to SurrealDB endpoint at '{}'. \n\
            Verify the server is actively running and accessible.\n\
            Underlying driver error: {}",
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

#[derive(Debug)]
pub enum BindError {
    NamespaceNotFound { requested: String, available: Vec<String> },
    DatabaseNotFound { requested: String, available: Vec<String> },
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindError::NamespaceNotFound { requested, available } => write!(
                f,
                "Namespace '{}' does not exist.\nAvailable namespaces: {}",
                requested,
                if available.is_empty() { "(none)".to_string() } else { available.join(", ") }
            ),
            BindError::DatabaseNotFound { requested, available } => write!(
                f,
                "Database '{}' does not exist in this namespace.\nAvailable databases: {}",
                requested,
                if available.is_empty() { "(none)".to_string() } else { available.join(", ") }
            ),
        }
    }
}

pub async fn use_namespace_and_db(
    db: &Surreal<Any>,
    ns: &str,
    db_name: &str,
) -> Result<(), BindError> {
    let namespaces = list_namespaces(db).await;
    if !namespaces.contains_key(ns) {
        return Err(BindError::NamespaceNotFound {
            requested: ns.to_string(),
            available: namespaces.keys().cloned().collect(),
        });
    }

    db.use_ns(ns).await.expect("Failed to bind namespace after existence check");

    let databases = list_databases(db).await;
    if !databases.contains_key(db_name) {
        return Err(BindError::DatabaseNotFound {
            requested: db_name.to_string(),
            available: databases.keys().cloned().collect(),
        });
    }

    db.use_db(db_name).await.expect("Failed to bind database after existence check");

    println!(
        "{} Bound to namespace '{}', database '{}'.",
        "✔".green(),
        ns.bold(),
        db_name.bold()
    );
    Ok(())
}

pub async fn connection() -> Surreal<Any> {
    let db = connect_cluster().await;
    let ns = require_var("NAMESPACE");
    let db_name = require_var("DATABASE_NAME");

    if let Err(e) = use_namespace_and_db(&db, &ns, &db_name).await {
        panic!("\n[Bind Error]: {}\n", e);
    }

    db
}