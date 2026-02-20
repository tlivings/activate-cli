use crate::database::models::{Project, ProjectState};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde_json;
use tabled::{builder::Builder, settings::Style};

/// Format projects as a colored table
pub fn format_table(projects: Vec<Project>) -> String {
    if projects.is_empty() {
        return "No projects tracked".to_string();
    }

    let mut builder = Builder::new();

    // Add header
    builder.push_record(["Name", "Path", "State", "Last Touched", "Git Origin"]);

    for project in projects {
        let state_colored = match project.state {
            ProjectState::Active => "active".green(),
            ProjectState::Inactive => "inactive".yellow(),
            ProjectState::Archived => "archived".bright_black(),
        };

        let last_touched = format_timestamp(project.last_touched.timestamp());
        let git_origin = project.git_origin.as_deref().unwrap_or("-");
        let path_str = project.path.to_string_lossy().to_string();

        builder.push_record([
            &project.name,
            &path_str,
            &state_colored.to_string(),
            &last_touched,
            git_origin,
        ]);
    }

    let mut table = builder.build();
    table.with(Style::modern());

    table.to_string()
}

/// Format projects as JSON
pub fn format_json(projects: Vec<Project>) -> String {
    serde_json::to_string_pretty(&projects).unwrap_or_else(|_| "[]".to_string())
}

/// Format projects as TSV (tab-separated values)
pub fn format_tsv(projects: Vec<Project>) -> String {
    if projects.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    // Header row
    output.push_str("Name\tPath\tState\tLast Touched\tGit Origin\n");

    for project in projects {
        let git_origin = project.git_origin.as_deref().unwrap_or("");

        // Escape tabs and newlines in field values
        let name = escape_tsv_field(&project.name);
        let path = escape_tsv_field(&project.path.to_string_lossy());
        let state = project.state.to_string();
        let last_touched = project.last_touched.timestamp().to_string();
        let git_origin = escape_tsv_field(git_origin);

        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            name, path, state, last_touched, git_origin
        ));
    }

    output
}

/// Convert Unix timestamp to human-readable relative time
pub fn format_timestamp(timestamp: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| Utc::now());

    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    let seconds = duration.num_seconds();
    if seconds < 0 {
        return "future".to_string();
    }

    if seconds < 60 {
        return "just now".to_string();
    }

    let minutes = duration.num_minutes();
    if minutes < 60 {
        return if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        };
    }

    let hours = duration.num_hours();
    if hours < 24 {
        return if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        };
    }

    let days = duration.num_days();
    if days < 30 {
        return if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        };
    }

    // For older dates, show the full date
    dt.format("%Y-%m-%d").to_string()
}

/// Escape tabs and newlines in TSV field values
fn escape_tsv_field(field: &str) -> String {
    field
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_project(name: &str, state: ProjectState) -> Project {
        Project {
            id: 1,
            name: name.to_string(),
            path: format!("/home/user/{}", name).into(),
            state,
            last_touched: Utc::now(),
            git_origin: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_format_table_empty() {
        let projects = vec![];
        let output = format_table(projects);
        assert_eq!(output, "No projects tracked");
    }

    #[test]
    fn test_format_table_with_projects() {
        let projects = vec![
            create_test_project("myapp", ProjectState::Active),
            create_test_project("library", ProjectState::Inactive),
        ];
        let output = format_table(projects);

        // Table should contain headers and project names
        assert!(output.contains("Name"));
        assert!(output.contains("Path"));
        assert!(output.contains("State"));
        assert!(output.contains("myapp"));
        assert!(output.contains("library"));
    }

    #[test]
    fn test_format_json_empty() {
        let projects = vec![];
        let output = format_json(projects);
        assert_eq!(output, "[]");
    }

    #[test]
    fn test_format_json_with_projects() {
        let projects = vec![create_test_project("myapp", ProjectState::Active)];
        let output = format_json(projects);

        // Should be valid JSON containing the project name
        assert!(output.contains("\"name\": \"myapp\"") || output.contains("\"name\":\"myapp\""));
        assert!(output.contains("\"state\": \"active\"") || output.contains("\"state\":\"active\""));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn test_format_tsv_empty() {
        let projects = vec![];
        let output = format_tsv(projects);
        assert_eq!(output, "");
    }

    #[test]
    fn test_format_tsv_with_projects() {
        let mut project = create_test_project("my\tapp", ProjectState::Active);
        project.git_origin = Some("https://github.com/user/repo.git".to_string());

        let projects = vec![project];
        let output = format_tsv(projects);

        // Should have header
        assert!(output.starts_with("Name\tPath\tState\tLast Touched\tGit Origin\n"));

        // Should escape tab in name
        assert!(output.contains("my\\tapp"));

        // Should contain git origin
        assert!(output.contains("https://github.com/user/repo.git"));
    }

    #[test]
    fn test_escape_tsv_field() {
        assert_eq!(escape_tsv_field("hello\tworld"), "hello\\tworld");
        assert_eq!(escape_tsv_field("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_tsv_field("normal text"), "normal text");
    }

    #[test]
    fn test_format_timestamp() {
        let now = Utc::now().timestamp();

        // Just now
        assert_eq!(format_timestamp(now), "just now");

        // 30 seconds ago
        assert_eq!(format_timestamp(now - 30), "just now");

        // 2 minutes ago
        assert_eq!(format_timestamp(now - 120), "2 minutes ago");

        // 1 hour ago
        assert_eq!(format_timestamp(now - 3600), "1 hour ago");

        // 2 days ago
        assert_eq!(format_timestamp(now - 172800), "2 days ago");

        // Old date (more than 30 days)
        let old_timestamp = now - (60 * 24 * 60 * 60); // 60 days ago
        let output = format_timestamp(old_timestamp);
        assert!(output.contains("-")); // Should be a date like "2024-12-20"
    }
}