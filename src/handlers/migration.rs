use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};
use crate::args::MigrateActions;
use crate::constants::{MIGRATIONS_TABLE, SCHEMA_ROOT};
use crate::core::connection;
use crate::utils::confirm;

#[derive(Debug, Clone)]
struct MigrationEntry {
    /// e.g. "20260925095558_user"
    id: String,
    schema_name: String,
    dir: PathBuf,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct AppliedRow {
    id: RecordId,
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
         DEFINE FIELD IF NOT EXISTS schema_name ON {MIGRATIONS_TABLE} TYPE string;
         DEFINE FIELD IF NOT EXISTS applied_at ON {MIGRATIONS_TABLE} TYPE datetime;"
    );

    if let Err(e) = db.query(&setup).await {
        eprintln!("{} Failed to ensure migrations table exists: {}", "✖".red(), e);
    }
}

fn discover_migrations() -> Vec<MigrationEntry> {
    let mut found = Vec::new();

    let Ok(schema_dirs) = fs::read_dir(SCHEMA_ROOT) else {
        return found;
    };

    for schema_entry in schema_dirs.flatten() {
        let schema_path = schema_entry.path();
        if !schema_path.is_dir() {
            continue;
        }
        let schema_name = schema_entry.file_name().to_string_lossy().to_string();

        let Ok(migration_dirs) = fs::read_dir(&schema_path) else {
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
                schema_name: schema_name.clone(),
                dir: migration_path,
            });
        }
    }

    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

async fn applied_migration_ids(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) -> Vec<String> {
    let query = format!("SELECT id FROM {MIGRATIONS_TABLE};");
    let mut res = match db.query(&query).await {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    res.take::<Vec<AppliedRow>>(0)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| match row.id.key {
            RecordIdKey::String(s) => Some(s),
            other => {
                eprintln!(
                    "{} Unexpected record id key shape in {MIGRATIONS_TABLE}: {:?}",
                    "⚠".yellow(),
                    other
                );
                None
            }
        })
        .collect()
}
pub async fn migrate_up() {
    let db = connection().await;
    ensure_migrations_table(&db).await;

    let all = discover_migrations();
    let applied = applied_migration_ids(&db).await;

    let pending: Vec<&MigrationEntry> = all
        .iter()
        .filter(|m| !applied.contains(&m.id))
        .collect();

    if pending.is_empty() {
        println!("{}", "Nothing to migrate -- already up to date.".bright_black().italic());
        return;
    }

    for migration in pending {
        let up_path = migration.dir.join("up.surql");
        let sql = match fs::read_to_string(&up_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{} Failed to read {}: {}", "✖".red(), up_path.display(), e);
                return;
            }
        };

        if sql.trim().is_empty() {
            println!(
                "{} {} has an empty up.surql, nothing to run but marking applied.",
                "⚠".yellow(),
                migration.id.cyan()
            );
        } else if let Err(e) = db.query(&sql).await {
            eprintln!(
                "{} Migration {} failed: {}\nStopping -- fix the error and re-run `migrate up`.",
                "✖".red(),
                migration.id.cyan(),
                e
            );
            return;
        }

        let record = "CREATE type::record($table, $migration_id) SET schema_name = $schema_name, applied_at = time::now();";
        if let Err(e) = db
            .query(record)
            .bind(("table", MIGRATIONS_TABLE))
            .bind(("migration_id", migration.id.clone()))
            .bind(("schema_name", migration.schema_name.clone()))
            .await
        {
            eprintln!("{} Ran migration but failed to record it: {}", "✖".red(), e);
            return;
        }

        println!("{} Applied {}", "✔".green(), migration.id.cyan());
    }
}
pub async fn migrate_down(force: bool) {
    let db = connection().await;
    ensure_migrations_table(&db).await;

    // 1. Look in the database: what's the most recently applied migration?
    let applied = applied_migration_ids(&db).await;
    if applied.is_empty() {
        println!("{}", "No applied migrations to roll back.".bright_black().italic());
        return;
    }
    let last_id = applied.iter().max().unwrap().clone();

    // 2. Look on disk: find that exact id's folder under migrations/schema/<schema_name>/<id>/
    let all = discover_migrations();
    let Some(migration) = all.iter().find(|m| m.id == last_id) else {
        eprintln!(
            "{} Migration {} is recorded as applied but its folder no longer exists on disk \
             (looked under {}/{}/). Can't safely run its down.surql -- resolve manually.",
            "✖".red(),
            last_id.cyan(),
            SCHEMA_ROOT,
            last_id
        );
        return;
    };

    // 3. Check whether down.surql actually exists for it.
    let down_path = migration.dir.join("down.surql");
    if !down_path.exists() {
        println!(
            "{} {} has no down.surql at all -- nothing to roll back.",
            "⚠".yellow(),
            migration.id.cyan()
        );
        if !force && !confirm("Mark it unapplied anyway, with no rollback run?") {
            println!("{}", "Aborted.".bright_black());
            return;
        }
    } else {
        let sql = fs::read_to_string(&down_path).unwrap_or_default();

        if sql.trim().is_empty() {
            println!(
                "{} {} has a blank down.surql -- rolling back will NOT undo anything in the \
                 database, but sudm will still forget it was applied unless you cancel.",
                "⚠".yellow(),
                migration.id.cyan()
            );
            if !force && !confirm("Mark it unapplied anyway?") {
                println!("{}", "Aborted.".bright_black());
                return;
            }
        } else if let Err(e) = db.query(&sql).await {
            eprintln!("{} Rollback failed: {}\nNot removing the migration record.", "✖".red(), e);
            return;
        }
    }

    // 4. Remove the record so `migrate up` will see it as pending again.
    let query = "DELETE type::record($table, $migration_id);";
    match db
        .query(query)
        .bind(("table", MIGRATIONS_TABLE))
        .bind(("migration_id", migration.id.clone()))
        .await
    {
        Ok(_) => println!(
            "{} Rolled back {} (schema: {})",
            "✔".green(),
            migration.id.cyan(),
            migration.schema_name.cyan()
        ),
        Err(e) => eprintln!("{} Ran down.surql but failed to remove its record: {}", "✖".red(), e),
    }
}