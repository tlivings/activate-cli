use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

pub mod schema;

/// Wrapper struct for the SQLite connection to make it easier to pass around
#[derive(Debug)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open a new database connection
    pub fn open() -> Result<Self> {
        let conn = open_database()?;
        Ok(Database { conn })
    }
}

/// Open the SQLite database, creating it if necessary
pub fn open_database() -> Result<Connection> {
    let db_path = get_database_path()?;

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create database directory")?;
    }

    // Open the database connection
    let conn = Connection::open(&db_path)
        .context(format!("Failed to open database at {:?}", db_path))?;

    // Configure SQLite pragmas for optimal performance and safety
    conn.pragma_update(None, "journal_mode", "WAL")
        .context("Failed to set WAL mode")?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("Failed to enable foreign keys")?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .context("Failed to set busy timeout")?;

    // Run migrations before returning
    schema::migrate(&conn)?;

    Ok(conn)
}

/// Get the path to the database file
fn get_database_path() -> Result<PathBuf> {
    // Use ~/.config/activate/ as the base directory
    let home = directories::BaseDirs::new()
        .context("Failed to determine home directory")?
        .home_dir()
        .to_path_buf();

    let config_dir = home.join(".config").join("activate");
    Ok(config_dir.join("db.sqlite"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_opens_successfully() {
        // This test will use a temporary directory instead of the real config dir
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let result = open_database();
        assert!(result.is_ok(), "Database should open successfully");

        // Verify the database file was created
        let db_path = temp_dir.path().join(".config").join("activate").join("db.sqlite");
        assert!(db_path.exists(), "Database file should be created");
    }
}