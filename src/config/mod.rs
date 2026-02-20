use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub mod paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory to track projects in (required - set on first run)
    pub tracked_directory: Option<PathBuf>,

    /// Patterns to ignore when scanning directories
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,

    /// Database file location (optional, defaults to data dir)
    pub database_path: Option<PathBuf>,
}

fn default_ignore_patterns() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        "__pycache__".to_string(),
        ".DS_Store".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tracked_directory: None, // Must be set by user
            ignore_patterns: default_ignore_patterns(),
            database_path: None,
        }
    }
}

impl Config {
    /// Load configuration from disk
    pub fn load() -> Result<Config> {
        load_config()
    }

    /// Save the configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_path = paths::get_config_file()?;
        let config_str =
            toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        fs::write(&config_path, config_str)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }

    /// Check if configuration is complete (tracked_directory is set)
    pub fn is_configured(&self) -> bool {
        self.tracked_directory.is_some()
    }

    /// Get tracked_directory, panics if not set
    pub fn tracked_directory(&self) -> &PathBuf {
        self.tracked_directory
            .as_ref()
            .expect("tracked_directory not configured")
    }
}

/// Load configuration from disk, creating default if it doesn't exist
pub fn load_config() -> Result<Config> {
    let config_path = paths::get_config_file()?;

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
    }

    // If config file doesn't exist, create it with defaults
    if !config_path.exists() {
        let default_config = Config::default();
        default_config
            .save()
            .context("Failed to save default configuration")?;
        return Ok(default_config);
    }

    // Load and parse existing config
    let config_str = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config from {:?}", config_path))?;

    let config = toml::from_str::<Config>(&config_str)
        .with_context(|| format!("Failed to parse config from {:?}", config_path))
        .unwrap_or_else(|_| {
            // If parsing fails, fall back to defaults
            eprintln!("Warning: Could not parse config file, using defaults");
            Config::default()
        });

    Ok(config)
}

/// Run first-time setup if tracked_directory is not set
pub fn run_first_time_setup() -> Result<Config> {
    let config_path = paths::get_config_file()?;

    println!("Welcome to activate!");
    println!();
    println!("First-time setup required.");
    println!();

    // Ask for tracked directory
    print!("Enter your projects directory (e.g., ~/Development): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        anyhow::bail!("Projects directory is required. Run 'activate --config' to set it.");
    }

    // Expand ~ to home directory
    let expanded = if input.starts_with("~/") {
        if let Some(home) = directories::UserDirs::new() {
            home.home_dir().join(&input[2..])
        } else {
            PathBuf::from(input)
        }
    } else if input == "~" {
        if let Some(home) = directories::UserDirs::new() {
            home.home_dir().to_path_buf()
        } else {
            PathBuf::from(input)
        }
    } else {
        PathBuf::from(input)
    };

    // Verify directory exists
    if !expanded.exists() {
        print!("Directory doesn't exist. Create it? [Y/n]: ");
        io::stdout().flush()?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        let confirm = confirm.trim().to_lowercase();

        if confirm.is_empty() || confirm == "y" || confirm == "yes" {
            fs::create_dir_all(&expanded)
                .with_context(|| format!("Failed to create directory: {:?}", expanded))?;
            println!("Created: {:?}", expanded);
        } else {
            anyhow::bail!("Setup cancelled. Run 'activate --config' to configure.");
        }
    }

    // Save config
    let mut config = Config::default();
    config.tracked_directory = Some(expanded.clone());
    config.save()?;

    println!();
    println!("Configuration saved to: {:?}", config_path);
    println!("Projects directory: {:?}", expanded);
    println!();
    println!("Run 'activate --sync' to discover existing projects.");

    Ok(config)
}

/// Open config file in editor
pub fn open_config_in_editor() -> Result<()> {
    let config_path = paths::get_config_file()?;

    // Ensure config file exists
    if !config_path.exists() {
        let default_config = Config::default();
        default_config.save()?;
    }

    // Get editor from environment
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Platform-specific fallbacks
            if cfg!(target_os = "macos") {
                "open -t".to_string()
            } else if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

    println!("Opening config: {:?}", config_path);

    // Parse editor command (might have flags like "open -t")
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (cmd, args) = parts.split_first().context("Invalid EDITOR")?;

    let status = std::process::Command::new(cmd)
        .args(args)
        .arg(&config_path)
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;

    if !status.success() {
        anyhow::bail!("Editor exited with error");
    }

    Ok(())
}
