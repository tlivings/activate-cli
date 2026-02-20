use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::database::operations::get_project_by_name;
use crate::database::Database;

/// Execute the status command - show detailed project information
pub fn execute_status(db: &Database, name: &str) -> Result<()> {
    let project = get_project_by_name(&db.conn, name)?
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", name))?;

    // Check if path exists
    let path_exists = project.path.exists();

    println!("Project: {}", project.name);
    println!("Path: {}", project.path.display());
    println!(
        "Exists: {}",
        if path_exists { "yes" } else { "NO - MISSING" }
    );
    println!("State: {}", project.state);
    println!(
        "Last touched: {}",
        format_relative_time(project.last_touched)
    );
    println!("Visit count: {}", project.visit_count);
    if let Some(origin) = &project.git_origin {
        println!("Git origin: {}", origin);
    }
    println!("Created: {}", project.created_at.format("%Y-%m-%d %H:%M"));

    Ok(())
}

/// Format a timestamp as a relative time string
fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if duration.num_days() < 7 {
        let days = duration.num_days();
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if duration.num_weeks() < 4 {
        let weeks = duration.num_weeks();
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{operations, schema};
    use chrono::Duration;
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn setup_test_db() -> Result<Database> {
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(Database { conn })
    }

    #[test]
    fn test_status_existing_project() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Status should succeed
        let result = execute_status(&db, "my-project");
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_not_found() {
        let db = setup_test_db().unwrap();
        let result = execute_status(&db, "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_format_relative_time_just_now() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now), "just now");
    }

    #[test]
    fn test_format_relative_time_minutes() {
        let past = Utc::now() - Duration::minutes(5);
        assert_eq!(format_relative_time(past), "5 minutes ago");
    }

    #[test]
    fn test_format_relative_time_hours() {
        let past = Utc::now() - Duration::hours(3);
        assert_eq!(format_relative_time(past), "3 hours ago");
    }

    #[test]
    fn test_format_relative_time_days() {
        let past = Utc::now() - Duration::days(2);
        assert_eq!(format_relative_time(past), "2 days ago");
    }

    #[test]
    fn test_format_relative_time_weeks() {
        let past = Utc::now() - Duration::weeks(2);
        assert_eq!(format_relative_time(past), "2 weeks ago");
    }

    #[test]
    fn test_status_case_insensitive() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("MyProject");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "MyProject", &project_path).unwrap();

        // Status with different case should work
        let result = execute_status(&db, "myproject");
        assert!(result.is_ok());
    }
}
