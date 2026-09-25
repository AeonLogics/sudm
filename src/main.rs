// module declaration
mod core;
mod args;
mod handlers;

// imports
use crate::core::connection;
use crate::args::{Args, Commands};
use clap::Parser;
use crate::handlers::{db_handler, ns_handler};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let args = Args::parse();

    match args.commands {
        Commands::Init => handlers::init_migrations(),
        Commands::Inspect => handlers::inspect_cluster().await,
        Commands::Ns { actions } => ns_handler(actions).await,
        Commands::Db { actions  } => db_handler(actions).await,
    }
}
