use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "activate")]
#[command(about = "Quick access to any tracked project", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a project to the database
    Add {
        /// Path to the project directory
        path: PathBuf,

        /// Optional name for the project (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Remove a project from the database
    Remove {
        /// Name or path of the project to remove
        identifier: String,
    },

    /// List all tracked projects
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Filter projects by state (active, archived, all)
        #[arg(short, long, default_value = "active")]
        state: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Display as a formatted table
    Table,
    /// Display as JSON
    Json,
    /// Display as tab-separated values
    Tsv,
}