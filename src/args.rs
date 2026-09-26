use clap::{Parser, Subcommand};
use clap::builder::Str;
use surrealdb::types::Action;

#[derive(Parser, Debug)]
#[command(
    name = "sudm",
    version,
    about = "Async SurrealDB migration CLI",
    long_about = "A dedicated CLI driver utility for managing connections and queries in SurrealDB."
)]
pub struct Args {
    #[arg(short, long, env = "DATABASE_URL")]
    pub url: String,
    #[arg(short = 'U', long, env = "SURREAL_USER")]
    pub user: String,
    #[arg(short = 'P', long, env = "SURREAL_PASS")]
    pub pass: String,
    #[arg(short, long, env = "NAMESPACE")]
    pub namespace: String,
    #[arg(short = 'd', long, env = "DATABASE_NAME")]
    pub database_name: String,

    #[command(subcommand)]
    pub commands: Commands,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    Init,
    Inspect {
        #[command(subcommand)]
        actions: InspectCommands,
    },
    Ns {
        #[command(subcommand)]
        actions: NSCommands
    },
    Db {
        #[command(subcommand)]
        actions: DBCommands
    },
    Schema {
        #[command(subcommand)]
        actions: SchemaActions,
    },
    Migrate {
        #[command(subcommand)]
        actions: MigrateActions,
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum InspectCommands {
    Tree,
    Raw
}

#[derive(Subcommand, Debug, Clone)]
pub enum NSCommands {
    New { name: String },
    Remove { name: String }
}

#[derive(Subcommand, Debug, Clone)]
pub enum DBCommands {
    New { ns_name: String, db_name: String },
    Remove { ns_name: String, db_name: String }
}

#[derive(Subcommand, Debug, Clone)]
pub enum SchemaActions {
    New {
        name: String,
    },
    Add {
        name: String,
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum MigrateActions {
    Up,
    Down {
        #[arg(long)]
        force: bool,
    },
}