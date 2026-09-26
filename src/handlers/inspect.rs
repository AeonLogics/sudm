use colored::Colorize;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use surrealdb::types::SurrealValue;
use crate::args::InspectCommands;
use crate::core::{connect_cluster, list_namespaces, list_databases};


pub async fn inspect_handler(actions: InspectCommands) {
    match actions {
        InspectCommands::Raw => inspect_cluster_raw().await,
        InspectCommands::Tree => inspect_cluster().await,
    }
}

#[derive(SurrealValue, Deserialize, Debug)]
struct DatabaseInfo {
    tables: BTreeMap<String, String>,
}

async fn list_tables(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) -> BTreeMap<String, String> {
    let mut res = match db.query("INFO FOR DB;").await {
        Ok(r) => r,
        Err(_) => return BTreeMap::new(),
    };
    res.take::<Option<DatabaseInfo>>(0)
        .ok()
        .flatten()
        .map(|info| info.tables)
        .unwrap_or_default()
}

pub async fn inspect_cluster() {
    let db = connect_cluster().await;
    println!("{}", "\n🔎 Analyzing SurrealDB Cluster Tree Structure...".cyan().bold());

    let namespaces = list_namespaces(&db).await;

    if namespaces.is_empty() {
        println!("└── {}", "No namespaces initialized in this cluster node.".bright_black().italic());
        return;
    }

    let ns_keys: Vec<&String> = namespaces.keys().collect();

    for (ns_idx, ns_name) in ns_keys.iter().enumerate() {
        let is_last_ns = ns_idx == ns_keys.len() - 1;
        let ns_prefix = if is_last_ns { "└──" } else { "├──" };

        println!(
            "{} 📦 Namespace: {} {}",
            ns_prefix.green(),
            ns_name.green().bold(),
            format!("(len={})", ns_name.len()).bright_black()
        );

        if db.use_ns(ns_name.to_string()).await.is_err() {
            let prefix = if is_last_ns { "    └──" } else { "│   └──" };
            println!("{} {}", prefix, "Authentication scope locked".red().italic());
            continue;
        }

        let databases = list_databases(&db).await;
        let db_keys: Vec<&String> = databases.keys().collect();
        let ns_branch = if is_last_ns { "    " } else { "│   " };

        if db_keys.is_empty() {
            println!("{}└── {}", ns_branch, "No inner databases initialized".bright_black().italic());
            continue;
        }

        for (db_idx, db_name) in db_keys.iter().enumerate() {
            let is_last_db = db_idx == db_keys.len() - 1;
            let db_branch = if is_last_db { "└──" } else { "├──" };

            println!(
                "{}{} 💾 Database: {} {}",
                ns_branch.green(),
                db_branch.green(),
                db_name.bright_blue(),
                format!("(len={})", db_name.len()).bright_black()
            );

            if db.use_db(db_name.to_string()).await.is_err() {
                let prefix = format!("{}{}", ns_branch, if is_last_db { "    └──" } else { "│   └──" });
                println!("{} {}", prefix, "Authentication scope locked".red().italic());
                continue;
            }

            let tables = list_tables(&db).await;
            let table_keys: Vec<&String> = tables.keys().collect();
            let db_line = format!("{}{}", ns_branch, if is_last_db { "    " } else { "│   " });

            if table_keys.is_empty() {
                println!("{}└── {}", db_line, "No tables defined".bright_black().italic());
            } else {
                for (t_idx, table_name) in table_keys.iter().enumerate() {
                    let is_last_t = t_idx == table_keys.len() - 1;
                    let t_branch = if is_last_t { "└──" } else { "├──" };
                    println!("{}{} 📄 Table: {}", db_line.green(), t_branch.green(), table_name.magenta());
                }
            }
        }
    }
    println!();
}


pub async fn inspect_cluster_raw() {
    let db = connect_cluster().await;
    println!("{}", "\n🔎 Full recursive cluster dump:\n".cyan().bold());

    let mut root_response = match db.query("INFO FOR ROOT;").await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} Failed to query cluster root metadata: {}", "✖".red(), e);
            return;
        }
    };

    let root_raw: Value = match root_response.take(0) {
        Ok(Some(v)) => v,
        Ok(None) => {
            println!("{}", "(empty ROOT response)".bright_black().italic());
            return;
        }
        Err(e) => {
            eprintln!("{} Failed to parse ROOT response: {}", "✖".red(), e);
            return;
        }
    };

    let namespaces = list_namespaces(&db).await;
    let mut tree = serde_json::Map::new();
    tree.insert("root".to_string(), root_raw);

    let mut ns_tree = serde_json::Map::new();

    for ns_name in namespaces.keys() {
        if db.use_ns(ns_name.to_string()).await.is_err() {
            ns_tree.insert(ns_name.clone(), serde_json::json!({ "error": "locked" }));
            continue;
        }

        let databases = list_databases(&db).await;
        let mut db_tree = serde_json::Map::new();

        for db_name in databases.keys() {
            if db.use_db(db_name.to_string()).await.is_err() {
                db_tree.insert(db_name.clone(), serde_json::json!({ "error": "locked" }));
                continue;
            }

            let mut db_response = match db.query("INFO FOR DB;").await {
                Ok(r) => r,
                Err(_) => {
                    db_tree.insert(db_name.clone(), serde_json::json!({ "error": "query failed" }));
                    continue;
                }
            };

            let db_raw = db_response
                .take::<Option<Value>>(0)
                .unwrap_or(None)
                .unwrap_or(Value::Null);
            db_tree.insert(db_name.clone(), db_raw);
        }

        ns_tree.insert(ns_name.clone(), Value::Object(db_tree));
    }

    tree.insert("namespaces_detail".to_string(), Value::Object(ns_tree));

    match serde_json::to_string_pretty(&Value::Object(tree)) {
        Ok(pretty) => println!("{}", pretty),
        Err(e) => eprintln!("{} Failed to format tree as JSON: {}", "✖".red(), e),
    }
    println!();
}