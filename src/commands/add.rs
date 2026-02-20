use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::database::operations::update_git_origin;
use crate::database::Database;
use crate::git::detect_origin;
use crate::utils::paths::canonicalize_project_path;

/// Execute the add command to track a new project
pub fn execute_add(db: &Database, path: &Path, name_override: Option<String>) -> Result<()> {
    // Check if the path exists
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Path does not exist: {}",
            path.display()
        ));
    }

    // Canonicalize the path
    let canonical_path = canonicalize_project_path(path)
        .context("Failed to canonicalize project path")?;

    // Extract project name
    let project_name = if let Some(name) = name_override {
        // Validate the provided name
        validate_project_name(&name)?;
        name
    } else {
        // Use the directory name as the project name
        canonical_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Could not extract project name from path"))?
            .to_string()
    };

    // Validate the extracted/provided name
    validate_project_name(&project_name)?;

    // Try to add the project to the database
    match crate::database::operations::add_project(&db.conn, &project_name, &canonical_path) {
        Ok(_) => {
            // Detect and store git origin if available
            if let Some(origin) = detect_origin(&canonical_path) {
                update_git_origin(&db.conn, &project_name, Some(&origin))?;
            }

            println!(
                "{} Added project '{}' at {}",
                "✓".green().bold(),
                project_name.cyan(),
                canonical_path.display().to_string().yellow()
            );
            Ok(())
        }
        Err(e) => {
            // Check for specific error types and provide helpful messages
            let error_msg = e.to_string();
            if error_msg.contains("already tracked") {
                Err(anyhow::anyhow!(
                    "Project already tracked at this path: {}",
                    canonical_path.display()
                ))
            } else if error_msg.contains("UNIQUE constraint failed: projects.name") {
                Err(anyhow::anyhow!(
                    "Project with name '{}' already exists. Use --name to specify a different name.",
                    project_name
                ))
            } else {
                Err(e).context("Failed to add project to database")
            }
        }
    }
}

/// Validate a project name
fn validate_project_name(name: &str) -> Result<()> {
    // Check for empty name
    if name.is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    // Check for slashes (would cause issues with subcommands)
    if name.contains('/') || name.contains('\\') {
        return Err(anyhow::anyhow!(
            "Project name cannot contain slashes: '{}'",
            name
        ));
    }

    // Check for reasonable length
    if name.len() > 100 {
        return Err(anyhow::anyhow!(
            "Project name is too long (max 100 characters): '{}'",
            name
        ));
    }

    // Check for whitespace-only names
    if name.trim().is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be only whitespace"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("valid-name").is_ok());
        assert!(validate_project_name("valid_name_123").is_ok());
        assert!(validate_project_name("Valid Project Name").is_ok());

        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("name/with/slash").is_err());
        assert!(validate_project_name("name\\with\\backslash").is_err());
        assert!(validate_project_name("   ").is_err());

        let long_name = "a".repeat(101);
        assert!(validate_project_name(&long_name).is_err());
    }
}