use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

mod cli;
mod config;
mod error;
mod utils;

use cli::{Cli, Commands, OutputFormat};
use config::load_config;

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration (creates if missing)
    let config = load_config().context("Failed to load configuration")?;

    // Ensure data directory exists
    let data_dir = config::paths::get_data_dir()?;
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create data directory at {:?}", data_dir))?;

    // Process commands
    match cli.command {
        Commands::Add { path: _, name: _ } => {
            todo!("Implement add command")
        }
        Commands::Remove { identifier: _ } => {
            todo!("Implement remove command")
        }
        Commands::List { format: _, state: _ } => {
            todo!("Implement list command")
        }
    }

    Ok(())
}