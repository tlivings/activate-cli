use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub mod paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory to track projects in (defaults to ~/Development)
    pub tracked_directory: PathBuf,

    /// Patterns to ignore when scanning directories
    pub ignore_patterns: Vec<String>,

    /// Database file location (optional, defaults to data dir)
    pub database_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let home = directories::UserDirs::new()
            .expect("Could not determine home directory")
            .home_dir()
            .to_path_buf();

        Self {
            tracked_directory: home.join("Development"),
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "__pycache__".to_string(),
                ".DS_Store".to_string(),
            ],
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
        let config_str = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        fs::write(&config_path, config_str)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
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
        default_config.save()
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