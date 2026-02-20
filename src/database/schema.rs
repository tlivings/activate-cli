use anyhow::{Context, Result};
use rusqlite::Connection;

/// Array of database migrations, indexed by version
pub const MIGRATIONS: &[&str] = &[
    // Version 1: Initial schema
    r#"
    -- Create schema version table to track migrations
    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY NOT NULL,
        applied_at INTEGER NOT NULL DEFAULT (unixepoch())
    );

    -- Create projects table with all required columns
    CREATE TABLE IF NOT EXISTS projects (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE COLLATE NOCASE,    -- Case-insensitive uniqueness
        path TEXT NOT NULL UNIQUE,                    -- Canonical paths only
        state TEXT NOT NULL CHECK(state IN ('active', 'inactive', 'archived')),
        last_touched INTEGER NOT NULL,                -- Unix timestamp
        git_origin TEXT,                               -- Nullable for Phase 3
        created_at INTEGER NOT NULL DEFAULT (unixepoch()),
        updated_at INTEGER NOT NULL DEFAULT (unixepoch())
    );

    -- Create indexes for query performance
    CREATE INDEX idx_projects_state ON projects(state);
    CREATE INDEX idx_projects_last_touched ON projects(last_touched);

    -- Create trigger to update updated_at on changes
    CREATE TRIGGER update_projects_updated_at
    AFTER UPDATE ON projects
    BEGIN
        UPDATE projects SET updated_at = unixepoch() WHERE id = NEW.id;
    END;
    "#,
    // Version 2: Add visit_count for frecency tracking
    r#"
    -- Add visit_count column for frecency scoring
    ALTER TABLE projects ADD COLUMN visit_count INTEGER NOT NULL DEFAULT 0;

    -- Create compound index for optimized frecency queries
    CREATE INDEX idx_projects_frecency ON projects(state, last_touched DESC, visit_count DESC);
    "#,
    // Version 3: Add ignored column for hiding projects from TUI
    r#"
    -- Add ignored column (0 = visible, 1 = hidden from TUI)
    ALTER TABLE projects ADD COLUMN ignored INTEGER NOT NULL DEFAULT 0;
    "#,
];

/// Run all pending database migrations
pub fn migrate(conn: &Connection) -> Result<()> {
    // Get current schema version
    let current_version = get_schema_version(conn)?;

    // Apply any pending migrations
    for (index, migration) in MIGRATIONS.iter().enumerate() {
        let version = index + 1; // Migrations are 1-indexed

        if version > current_version {
            conn.execute_batch(migration)
                .context(format!("Failed to run migration version {}", version))?;

            set_schema_version(conn, version)?;

            println!("Applied migration version {}", version);
        }
    }

    Ok(())
}

/// Get the current schema version from the database
fn get_schema_version(conn: &Connection) -> Result<usize> {
    // First, check if the schema_version table exists
    let table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .context("Failed to check if schema_version table exists")?;

    if table_exists == 0 {
        // Table doesn't exist yet, we're at version 0
        return Ok(0);
    }

    // Get the maximum version from the schema_version table
    let version: Option<i32> = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get(0)
        })
        .context("Failed to get schema version")?;

    Ok(version.unwrap_or(0) as usize)
}

/// Set the schema version in the database
fn set_schema_version(conn: &Connection, version: usize) -> Result<()> {
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [version as i32],
    )
    .context(format!("Failed to set schema version to {}", version))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_run_successfully() {
        // Create an in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations
        let result = migrate(&conn);
        assert!(result.is_ok(), "Migrations should run successfully");

        // Verify schema version was set
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 3, "Schema version should be 3 after migration");

        // Verify tables were created
        let projects_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(projects_exists, 1, "Projects table should exist");

        // Verify indexes were created
        let index_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='projects'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            index_count >= 2,
            "At least 2 indexes should exist on projects table"
        );
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice
        migrate(&conn).unwrap();
        let result = migrate(&conn);

        assert!(result.is_ok(), "Running migrations twice should not fail");

        // Version should still be 3
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 3, "Schema version should remain 3");
    }

    #[test]
    fn test_state_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        // Test valid states
        for state in &["active", "inactive", "archived"] {
            let result = conn.execute(
                "INSERT INTO projects (name, path, state, last_touched) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    format!("test_{}", state),
                    format!("/path/{}", state),
                    state,
                    1234567890
                ],
            );
            assert!(result.is_ok(), "Should accept state: {}", state);
        }

        // Test invalid state
        let result = conn.execute(
            "INSERT INTO projects (name, path, state, last_touched) VALUES ('invalid', '/invalid', 'invalid_state', 1234567890)",
            [],
        );
        assert!(result.is_err(), "Should reject invalid state");
    }

    #[test]
    fn test_name_case_insensitive_uniqueness() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        // Insert a project
        conn.execute(
            "INSERT INTO projects (name, path, state, last_touched) VALUES ('TestProject', '/test', 'active', 1234567890)",
            [],
        ).unwrap();

        // Try to insert with different case
        let result = conn.execute(
            "INSERT INTO projects (name, path, state, last_touched) VALUES ('testproject', '/test2', 'active', 1234567890)",
            [],
        );
        assert!(
            result.is_err(),
            "Should reject duplicate name with different case"
        );
    }

    #[test]
    fn test_visit_count_column_exists() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        // Insert a project and verify visit_count defaults to 0
        conn.execute(
            "INSERT INTO projects (name, path, state, last_touched) VALUES ('test', '/test', 'active', 1234567890)",
            [],
        ).unwrap();

        let visit_count: i32 = conn
            .query_row(
                "SELECT visit_count FROM projects WHERE name = 'test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(visit_count, 0, "visit_count should default to 0");

        // Verify we can update visit_count
        conn.execute(
            "UPDATE projects SET visit_count = 5 WHERE name = 'test'",
            [],
        )
        .unwrap();

        let updated_count: i32 = conn
            .query_row(
                "SELECT visit_count FROM projects WHERE name = 'test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(updated_count, 5, "visit_count should be updatable");
    }

    #[test]
    fn test_frecency_index_exists() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        // Verify the frecency index exists
        let index_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_projects_frecency'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_exists, 1, "idx_projects_frecency index should exist");
    }
}
