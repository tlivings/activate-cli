use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::process;

mod automation;
mod cli;
mod commands;
mod config;
mod database;
mod error;
mod git;
mod navigation;
mod output;
mod shell;
mod tui;
mod utils;

use automation::trigger_demotion_check;
use cli::Cli;
use config::load_config;
use database::{get_database_path, Database};

fn main() -> Result<()> {
    // Trigger background demotion check (non-blocking)
    if let Ok(db_path) = get_database_path() {
        trigger_demotion_check(db_path);
    }

    let cli = Cli::parse();

    let _config = load_config().context("Failed to load configuration")?;

    let data_dir = config::paths::get_data_dir()?;
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create data directory at {:?}", data_dir))?;

    let db = Database::open().context("Failed to open database")?;

    // Dispatch based on flags (order matters - check flags before positional)
    let result = if cli.list {
        commands::list::execute_list(&db, cli.state.as_deref(), cli.json, cli.paths, cli.verbose)
    } else if let Some(ref path) = cli.add {
        commands::add::execute_add(&db, path, None)
    } else if let Some(ref name) = cli.remove {
        commands::remove::execute_remove(&db, name)
    } else if cli.sync {
        commands::sync::execute_sync(&db)
    } else if let Some(ref name) = cli.status {
        commands::status::execute_status(&db, name)
    } else if let Some(ref name) = cli.deactivate {
        commands::deactivate::execute_deactivate(&db, name)
    } else if let Some(ref name) = cli.archive {
        commands::archive::execute_archive(&db, name)
    } else if let Some(ref shell) = cli.init {
        commands::init::execute_init(shell)
    } else if let Some(ref name) = cli.query {
        commands::query::execute_query(&db, std::slice::from_ref(name), cli.exclude.as_deref())
    } else if let Some(ref shell) = cli.completions {
        commands::completions::execute_completions(&db, shell, cli.current.as_deref())
    } else if let Some(ref name) = cli.name {
        // Positional argument = activate project
        commands::activate::execute_activate(&db, name)
    } else {
        // No args = interactive TUI
        commands::interactive::execute_interactive(&db)
    };

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            // Print error chain for additional context
            for cause in e.chain().skip(1) {
                eprintln!("  {} {}", "^".yellow(), cause);
            }
            process::exit(1);
        }
    }
}
