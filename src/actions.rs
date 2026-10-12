use clap::{Parser, Subcommand};

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
    Add {
        name: String,
    },
    Run {
        #[command(subcommand)]
        actions: RunActions,
    }
}

#[derive(Subcommand, Debug)]
pub enum RunActions {
    #[command(name = "setup", alias = "s")]
    Setup,
    #[command(name = "migration", visible_alias = "mig", alias = "m")]
    Migration,
}