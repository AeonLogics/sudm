use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "surreal-cli-tool",
    version,
    about = "My Terminal Tool",
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
    Inspect,
    Ns {
        #[command(subcommand)]
        actions: NSCommands
    },
    Db {
        #[command(subcommand)]
        actions: DBCommands
    }
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
