// module declaration
mod core;
mod actions;
mod handlers;
mod utils;
mod constants;

// imports
use crate::actions::{Args, Commands};
use clap::Parser;
use crate::handlers::{add_handler, run_handler};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let args = Args::parse();

    match args.commands {
        Commands::Run { actions } => run_handler(actions).await,
        Commands::Add { name } => add_handler(name),
    }
}
