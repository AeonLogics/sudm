use colored::Colorize;
use chrono::Utc;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};
use crate::args::MigrateActions;
use crate::constants::{MIGRATIONS_TABLE, SCHEMA_ROOT};
use crate::core::connection;
use crate::utils::confirm;

#[derive(Debug, Clone)]
struct MigrationEntry {
    id: String,
    group_name: String,
    dir: PathBuf,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct AppliedRow {
    id: RecordId,
    batch_id: String,
    seq: i64,
}

pub async fn migration_handler(actions: MigrateActions) {
    match actions {
        MigrateActions::Up => migrate_up().await,
        MigrateActions::Down { force } => migrate_down(force).await,
    }
}

async fn ensure_migrations_table(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) {
    let setup = format!(
        "DEFINE TABLE IF NOT EXISTS {MIGRATIONS_TABLE} SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS group_name ON {MIGRATIONS_TABLE} TYPE string;
         DEFINE FIELD IF NOT EXISTS applied_at ON {MIGRATIONS_TABLE} TYPE datetime;
         DEFINE FIELD IF NOT EXISTS batch_id ON {MIGRATIONS_TABLE} TYPE string;
         DEFINE FIELD IF NOT EXISTS seq ON {MIGRATIONS_TABLE} TYPE int;"
    );

    if let Err(e) = db.query(&setup).await {
        eprintln!("{} Failed to ensure migrations table exists: {}", "✖".red(), e);
    }
}

fn discover_migrations() -> Vec<MigrationEntry> {
    let mut found = Vec::new();

    let Ok(group_dirs) = fs::read_dir(SCHEMA_ROOT) else {
        return found;
    };

    for group_entry in group_dirs.flatten() {
        let group_path = group_entry.path();
        if !group_path.is_dir() {
            continue;
        }
        let group_name = group_entry.file_name().to_string_lossy().to_string();

        let Ok(migration_dirs) = fs::read_dir(&group_path) else {
            continue;
        };

        for migration_entry in migration_dirs.flatten() {
            let migration_path = migration_entry.path();
            if !migration_path.is_dir() {
                continue;
            }

            let id = migration_entry.file_name().to_string_lossy().to_string();
            found.push(MigrationEntry {
                id,
                group_name: group_name.clone(),
                dir: migration_path,
            });
        }
    }

    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

async fn applied_rows(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) -> Vec<(String, String, i64)> {
    let query = format!("SELECT id, batch_id, seq FROM {MIGRATIONS_TABLE};");
    let mut res = match db.query(&query).await {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    res.take::<Vec<AppliedRow>>(0)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| match row.id.key {
            RecordIdKey::String(s) => Some((s, row.batch_id, row.seq)),
            other => {
                eprintln!("{} Unexpected record id shape: {:?}", "⚠".yellow(), other);
                None
            }
        })
        .collect()
}

fn read_step(path: &PathBuf, label: &str, migration_id: &str) -> Result<Option<String>, ()> {
    match fs::read_to_string(path) {
        Ok(content) if content.trim().is_empty() => Ok(None),
        Ok(content) => Ok(Some(content)),
        Err(e) => {
            eprintln!("{} [{}] Failed to read {} for {}: {}", "✖".red(), label, path.display(), migration_id, e);
            Err(())
        }
    }
}

pub async fn migrate_up() {
    let db = connection().await;
    ensure_migrations_table(&db).await;

    let all = discover_migrations();
    let applied: Vec<String> = applied_rows(&db).await.into_iter().map(|(id, _, _)| id).collect();

    let pending: Vec<&MigrationEntry> = all.iter().filter(|m| !applied.contains(&m.id)).collect();

    if pending.is_empty() {
        println!("{}", "Nothing to migrate -- already up to date.".bright_black().italic());
        return;
    }

    let batch_id = Utc::now().format("batch_%Y%m%d%H%M%S%3f").to_string();
    let mut seq: i64 = 0;
    let mut applied_count = 0usize;

    'outer: for migration in &pending {
        let steps = [
            (format!("{}-schema.surql", migration.group_name), "schema"),
            (format!("{}-functions.surql", migration.group_name), "functions"),
            (format!("{}-events.surql", migration.group_name), "events"),
        ];

        for (filename, label) in &steps {
            let path = migration.dir.join(filename);
            let sql = match read_step(&path, label, &migration.id) {
                Ok(Some(s)) => s,
                Ok(None) => continue,
                Err(()) => break 'outer,
            };

            if let Err(e) = db.query(&sql).await {
                eprintln!(
                    "{} [{}] Migration {} failed: {}\nStopping -- fix the error and re-run `migrate up`.",
                    "✖".red(), label, migration.id.cyan(), e
                );
                break 'outer;
            }
        }

        let record = "CREATE type::record($table, $migration_id) \
                       SET group_name = $group_name, applied_at = time::now(), \
                           batch_id = $batch_id, seq = $seq;";
        if let Err(e) = db
            .query(record)
            .bind(("table", MIGRATIONS_TABLE))
            .bind(("migration_id", migration.id.clone()))
            .bind(("group_name", migration.group_name.clone()))
            .bind(("batch_id", batch_id.clone()))
            .bind(("seq", seq))
            .await
        {
            eprintln!("{} Ran migration but failed to record it: {}", "✖".red(), e);
            break;
        }

        println!("{} Applied {}", "✔".green(), migration.id.cyan());
        seq += 1;
        applied_count += 1;
    }

    if applied_count > 0 {
        println!("\n{} Applied {} migration(s) in batch {}", "✔".green().bold(), applied_count, batch_id.cyan());
    }
}

pub async fn migrate_down(force: bool) {
    let db = connection().await;
    ensure_migrations_table(&db).await;

    let applied = applied_rows(&db).await;
    if applied.is_empty() {
        println!("{}", "No applied migrations to roll back.".bright_black().italic());
        return;
    }

    let last_batch = applied.iter().map(|(_, batch, _)| batch.clone()).max().unwrap();

    let mut batch_entries: Vec<(String, i64)> = applied
        .into_iter()
        .filter(|(_, batch, _)| *batch == last_batch)
        .map(|(id, _, seq)| (id, seq))
        .collect();

    batch_entries.sort_by_key(|(_, seq)| *seq);
    batch_entries.reverse();

    let batch_ids: Vec<String> = batch_entries.into_iter().map(|(id, _)| id).collect();

    println!("{} Rolling back batch {} ({} migration(s)):", "⚠".yellow().bold(), last_batch.cyan(), batch_ids.len());
    for id in &batch_ids {
        println!("    - {}", id.cyan());
    }

    if !force && !confirm(&format!("Roll back all {} migration(s) in this batch?", batch_ids.len())) {
        println!("{}", "Aborted.".bright_black());
        return;
    }

    let all = discover_migrations();
    let mut rolled_back = 0usize;

    for id in &batch_ids {
        let Some(migration) = all.iter().find(|m| &m.id == id) else {
            eprintln!("{} Migration {} no longer exists on disk. Skipping -- resolve manually.", "✖".red(), id.cyan());
            continue;
        };

        let down_path = migration.dir.join("rollback.surql");
        let mut ok_to_delete = true;

        if !down_path.exists() {
            println!("{} {} has no rollback.surql -- nothing to roll back.", "⚠".yellow(), migration.id.cyan());
            if !force && !confirm("Mark it unapplied anyway, with no rollback run?") {
                println!("{} Skipping {}, leaving it recorded as applied.", "⚠".yellow(), migration.id.cyan());
                ok_to_delete = false;
            }
        } else {
            let sql = fs::read_to_string(&down_path).unwrap_or_default();

            if sql.trim().is_empty() {
                println!(
                    "{} {} has a blank rollback.surql -- nothing will actually be undone, \
                     but sudm will still forget it was applied unless you cancel.",
                    "⚠".yellow(), migration.id.cyan()
                );
                if !force && !confirm("Mark it unapplied anyway?") {
                    println!("{} Skipping {}, leaving it recorded as applied.", "⚠".yellow(), migration.id.cyan());
                    ok_to_delete = false;
                }
            } else if let Err(e) = db.query(&sql).await {
                eprintln!(
                    "{} Rollback failed for {}: {}\nStopping here -- not removing this or later records.",
                    "✖".red(), migration.id.cyan(), e
                );
                break;
            }
        }

        if ok_to_delete {
            let query = "DELETE type::record($table, $migration_id);";
            match db
                .query(query)
                .bind(("table", MIGRATIONS_TABLE))
                .bind(("migration_id", migration.id.clone()))
                .await
            {
                Ok(_) => {
                    println!("{} Rolled back {}", "✔".green(), migration.id.cyan());
                    rolled_back += 1;
                }
                Err(e) => eprintln!("{} Ran rollback but failed to remove record for {}: {}", "✖".red(), migration.id.cyan(), e),
            }
        }
    }

    println!(
        "\n{} Rolled back {} of {} migration(s) in batch {}",
        "✔".green().bold(), rolled_back, batch_ids.len(), last_batch.cyan()
    );
}