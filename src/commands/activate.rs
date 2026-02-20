use anyhow::{Context, Result};

use crate::config::Config;
use crate::database::models::ProjectState;
use crate::database::operations::{
    add_project_with_state, get_project_by_name, increment_visit_and_touch, list_projects,
    update_git_origin, update_project_state,
};
use crate::database::Database;
use crate::git::{clone_repository, detect_origin, extract_repo_name};
use crate::navigation::ProjectMatcher;

/// Execute the activate command - find or create project, mark active, output path
///
/// This is the primary interface. It:
/// 1. If input is a URL (HTTPS or SSH): clone and activate
/// 2. Otherwise searches for project by name (exact then fuzzy match)
/// 3. If found: marks active, increments visit, outputs path
/// 4. If not found: creates in tracked_directory and activates
pub fn execute_activate(db: &Database, name: &str) -> Result<()> {
    // Check if input looks like a URL (HTTPS or SSH format per CONTEXT.md)
    if name.starts_with("https://") || name.starts_with("git@") || name.starts_with("ssh://") {
        return clone_and_activate(db, name);
    }

    let projects = list_projects(&db.conn, None)?;
    let matcher = ProjectMatcher::new();

    // Try exact match first (highest priority)
    if let Some(project) = matcher.find_exact(name, &projects) {
        activate_existing(db, &project.name)?;
        println!("{}", project.path.display());
        return Ok(());
    }

    // Try fuzzy match
    let results = matcher.match_projects(name, &projects);
    if let Some(best) = results.first() {
        activate_existing(db, &best.project.name)?;
        println!("{}", best.project.path.display());
        return Ok(());
    }

    // No match - create new project
    create_new_project(db, name)
}

/// Activate an existing project: update state and increment visit
fn activate_existing(db: &Database, name: &str) -> Result<()> {
    update_project_state(&db.conn, name, ProjectState::Active)?;
    increment_visit_and_touch(&db.conn, name)?;

    // Update origin on activate if not already stored or changed
    if let Some(project) = get_project_by_name(&db.conn, name)? {
        if let Some(origin) = detect_origin(&project.path) {
            if project.git_origin.as_ref() != Some(&origin) {
                update_git_origin(&db.conn, name, Some(&origin))?;
            }
        }
    }

    Ok(())
}

/// Clone a repository from URL and activate it
fn clone_and_activate(db: &Database, url: &str) -> Result<()> {
    let config = Config::load()?;

    let repo_name = extract_repo_name(url)
        .ok_or_else(|| anyhow::anyhow!("Could not extract repository name from URL"))?;

    // Per CONTEXT.md: clone to tracked_directory/<repo-name>
    let dest = config.tracked_directory().join(&repo_name);

    // Per CONTEXT.md: error and abort if folder exists
    if dest.exists() {
        return Err(anyhow::anyhow!(
            "Folder '{}' already exists. Use `activate {}` to activate existing project.",
            dest.display(),
            repo_name
        ));
    }

    // Check if name already in database
    if get_project_by_name(&db.conn, &repo_name)?.is_some() {
        return Err(anyhow::anyhow!(
            "Project '{}' already exists in database.",
            repo_name
        ));
    }

    eprintln!("Cloning {} into {}...", url, dest.display());
    clone_repository(url, &dest)?;

    // Canonicalize path
    let canonical = dest
        .canonicalize()
        .context("Failed to canonicalize cloned path")?;

    // Add to database as active with origin
    add_project_with_state(&db.conn, &repo_name, &canonical, ProjectState::Active)?;
    update_git_origin(&db.conn, &repo_name, Some(url))?;
    increment_visit_and_touch(&db.conn, &repo_name)?;

    eprintln!("Cloned and activated '{}'", repo_name);
    println!("{}", canonical.display());

    Ok(())
}

/// Create a new project in tracked_directory
fn create_new_project(db: &Database, name: &str) -> Result<()> {
    // Load config to get tracked directory
    let config = Config::load()?;
    let new_path = config.tracked_directory().join(name);

    // Create directory
    std::fs::create_dir_all(&new_path)
        .with_context(|| format!("Failed to create directory: {}", new_path.display()))?;

    // Canonicalize the path
    let canonical = new_path
        .canonicalize()
        .context("Failed to canonicalize new project path")?;

    // Add to database as active
    add_project_with_state(&db.conn, name, &canonical, ProjectState::Active)?;
    increment_visit_and_touch(&db.conn, name)?;

    eprintln!("Created project '{}' at {}", name, canonical.display());
    println!("{}", canonical.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{operations, schema};
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn setup_test_db() -> Result<Database> {
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(Database { conn })
    }

    #[test]
    fn test_activate_existing_project() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Activate should succeed
        activate_existing(&db, "my-project").unwrap();

        // State should be active
        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Active);
        assert_eq!(project.visit_count, 1);
    }

    #[test]
    fn test_activate_increments_visit() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Activate multiple times
        activate_existing(&db, "my-project").unwrap();
        activate_existing(&db, "my-project").unwrap();
        activate_existing(&db, "my-project").unwrap();

        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.visit_count, 3);
    }

    #[test]
    fn test_activate_sets_state_to_active() {
        let db = setup_test_db().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let project_path = temp_dir.path().join("my-project");
        std::fs::create_dir(&project_path).unwrap();
        operations::add_project(&db.conn, "my-project", &project_path).unwrap();

        // Initially inactive
        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Inactive);

        activate_existing(&db, "my-project").unwrap();

        // Now active
        let project = operations::get_project_by_name(&db.conn, "my-project")
            .unwrap()
            .unwrap();
        assert_eq!(project.state, ProjectState::Active);
    }
}
