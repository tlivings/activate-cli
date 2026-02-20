use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

/// Get the configuration directory for the application
/// - Linux: ~/.config/activate/
/// - macOS: ~/Library/Application Support/activate/
/// - Windows: %APPDATA%\activate\
pub fn get_config_dir() -> Result<PathBuf> {
    ProjectDirs::from("", "", "activate")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .context("Could not determine configuration directory")
}

/// Get the data directory for the application
/// - Linux: ~/.local/share/activate/
/// - macOS: ~/Library/Application Support/activate/
/// - Windows: %APPDATA%\activate\
pub fn get_data_dir() -> Result<PathBuf> {
    ProjectDirs::from("", "", "activate")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .context("Could not determine data directory")
}

/// Get the path to the configuration file
pub fn get_config_file() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.toml"))
}

/// Get the path to the SQLite database file
pub fn get_database_file() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("projects.db"))
}