use colored::Colorize;
use serde::Deserialize;
use std::collections::BTreeMap;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;
use crate::core::connection;

#[derive(Deserialize, Debug, SurrealValue)]
struct RootInfo {
    namespaces: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct NamespaceInfo {
    databases: BTreeMap<String, String>,
}

pub async fn inspect_cluster() {
    let db = connection().await;
    println!("{}", "\n🔎 Analyzing SurrealDB Cluster Tree Structure...".cyan().bold());

    let mut root_response = match db.query("INFO FOR ROOT;").await {
        Ok(res) => res,
        Err(_) => {
            println!("  {} Failed to query cluster root metadata.", "✖".red());
            return;
        }
    };

    let namespaces = root_response
        .take::<Option<RootInfo>>(0)
        .unwrap()
        .map(|info| info.namespaces)
        .unwrap_or_default();

    if namespaces.is_empty() {
        println!("└── {}", "No namespaces initialized in this cluster node.".bright_black().italic());
        return;
    }

    let ns_keys: Vec<&String> = namespaces.keys().collect();

    for (ns_idx, ns_name) in ns_keys.iter().enumerate() {
        let is_last_ns = ns_idx == ns_keys.len() - 1;

        let ns_prefix = if is_last_ns { "└──" } else { "├──" };

        println!("{} 📦 Namespace: {}", ns_prefix.green(), ns_name.green().bold());

        if db.use_ns(ns_name.to_string()).await.is_err() {
            let db_prefix = if is_last_ns { "    └──" } else { "│   └──" };
            println!("{} {}", db_prefix, "Authentication scope locked".red().italic());
            continue;
        }

        let mut ns_response = match db.query("INFO FOR NS;").await {
            Ok(res) => res,
            Err(_) => continue,
        };

        let databases = ns_response
            .take::<Option<NamespaceInfo>>(0)
            .unwrap()
            .map(|info| info.databases)
            .unwrap_or_default();

        let db_keys: Vec<&String> = databases.keys().collect();
        let parent_line = if is_last_ns { "    " } else { "│   " };

        if db_keys.is_empty() {
            println!("{}└── {}", parent_line, "No inner databases initialized".bright_black().italic());
        } else {
            // 3. Print inner database leaves
            for (db_idx, db_name) in db_keys.iter().enumerate() {
                let is_last_db = db_idx == db_keys.len() - 1;
                let db_branch = if is_last_db { "└──" } else { "├──" };

                println!(
                    "{}{} 💾 Database: {}",
                    parent_line.green(),
                    db_branch.green(),
                    db_name.bright_blue()
                );
            }
        }
    }
    println!();
}
