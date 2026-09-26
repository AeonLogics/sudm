// module declaration
mod core;
mod args;
mod handlers;
mod utils;
mod constants;

// imports
use crate::args::{Args, Commands};
use clap::Parser;
use crate::handlers::{db_handler, inspect_handler, migration_handler, ns_handler, schema_handler};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let args = Args::parse();

    match args.commands {
        Commands::Init => handlers::init_migrations(),
        Commands::Inspect {actions } => inspect_handler(actions).await,
        Commands::Ns { actions } => ns_handler(actions).await,
        Commands::Db { actions  } => db_handler(actions).await,
        Commands::Schema { actions } => schema_handler(actions).await,
        Commands::Migrate {actions} => migration_handler(actions).await,
    }
}
