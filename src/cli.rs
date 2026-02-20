use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "activate")]
#[command(about = "Quick access to any tracked project", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Project name to navigate to (positional argument)
    pub name: Option<String>,

    // === Management Flags ===
    /// List all tracked projects
    #[arg(short, long)]
    pub list: bool,

    /// Add a project by path
    #[arg(short, long, value_name = "PATH")]
    pub add: Option<PathBuf>,

    /// Remove a project by name
    #[arg(short, long, value_name = "NAME")]
    pub remove: Option<String>,

    /// Synchronize project states
    #[arg(short, long)]
    pub sync: bool,

    /// Show project status
    #[arg(long, value_name = "NAME")]
    pub status: Option<String>,

    /// Deactivate a project
    #[arg(short, long, value_name = "NAME")]
    pub deactivate: Option<String>,

    /// Archive a project
    #[arg(long, value_name = "NAME")]
    pub archive: Option<String>,

    /// Initialize shell integration
    #[arg(long, value_name = "SHELL")]
    pub init: Option<String>,

    /// Query for project path (used by shell wrapper)
    #[arg(short, long, value_name = "NAME")]
    pub query: Option<String>,

    /// Generate completions (internal use)
    #[arg(long, value_name = "SHELL", hide = true)]
    pub completions: Option<String>,

    // === Modifiers ===
    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Show full paths (for list)
    #[arg(long)]
    pub paths: bool,

    /// Filter by state (for list): active, inactive, archived
    #[arg(long, value_name = "STATE")]
    pub state: Option<String>,

    /// Verbose output (show git status)
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Exclude directory from results (for query)
    #[arg(long, value_name = "PATH", hide = true)]
    pub exclude: Option<String>,

    /// Current completion word (for completions)
    #[arg(long, value_name = "WORD", hide = true)]
    pub current: Option<String>,
}
