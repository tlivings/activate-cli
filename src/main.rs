use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::process;

mod cli;
mod commands;
mod config;
mod database;
mod error;
mod navigation;
mod output;
mod utils;

use cli::{Cli, Commands};
use config::load_config;
use database::Database;

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration (creates if missing)
    let _config = load_config().context("Failed to load configuration")?;

    // Ensure data directory exists
    let data_dir = config::paths::get_data_dir()?;
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create data directory at {:?}", data_dir))?;

    // Open database connection
    let db = Database::open().context("Failed to open database")?;

    // Process commands
    let result = match cli.command {
        Commands::Add { path, name } => {
            commands::add::execute_add(&db, &path, name)
        }
        Commands::Remove { identifier } => {
            commands::remove::execute_remove(&db, &identifier)
        }
        Commands::List { format, state } => {
            commands::list::execute_list(&db, state.as_deref(), format)
        }
        Commands::Query { keywords, exclude } => {
            commands::query::execute_query(&db, &keywords, exclude.as_deref())
        }
    };

    // Handle command results with proper exit codes
    match result {
        Ok(()) => {
            process::exit(0);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);

            // Print error chain for additional context
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("  {} {}", "↳".yellow(), err);
                source = err.source();
            }

            process::exit(1);
        }
    }
}