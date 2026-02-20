use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "activate")]
#[command(about = "Quick access to any tracked project", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all tracked projects
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Filter projects by state (active, inactive, archived)
        #[arg(short, long)]
        state: Option<String>,
    },

    /// Query for a project path (used by shell wrapper)
    Query {
        /// Keywords to match against project names
        #[arg(required = true)]
        keywords: Vec<String>,

        /// Exclude the current directory from results
        #[arg(long)]
        exclude: Option<String>,
    },

    /// Activate a project (find or create, mark active, cd to it)
    Activate {
        /// Project name to activate
        name: String,
    },

    /// Deactivate a project (mark as inactive)
    Deactivate {
        /// Project name to deactivate
        name: String,
    },

    /// Archive a project (mark as archived)
    Archive {
        /// Project name to archive
        name: String,
    },

    /// Show detailed project status
    Status {
        /// Project name to show status for
        name: String,
    },

    /// Initialize shell integration
    Init {
        /// Shell type (bash, zsh, fish)
        shell: String,
    },

    /// Generate completions for shell (internal use)
    Completions {
        /// Shell type
        shell: String,

        /// Current word being completed
        #[arg(long)]
        current: Option<String>,
    },

    /// Synchronize project states (discover new, demote stale, check missing)
    Sync,
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