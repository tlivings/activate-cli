use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;

use crate::automation::demotion::perform_demotion_check;
use crate::automation::discovery::{add_discovered_projects, discover_new_projects};
use crate::config::Config;
use crate::database::operations::{list_projects, remove_project, update_git_origin};
use crate::database::Database;
use crate::git::detect_origin;

pub fn execute_sync(db: &Database) -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Syncing project states...".bold());

    // 1. Run demotion check (synchronously for feedback)
    let db_path = db.conn.path().map(PathBuf::from);
    if let Some(path) = db_path {
        let demoted = perform_demotion_check(&path)?;
        if demoted > 0 {
            println!(
                "  {} {} project(s) demoted to inactive (>14 days stale)",
                "->".yellow(),
                demoted
            );
        }
    }

    // 2. Discover new projects
    let discovered = discover_new_projects(db, &config)?;
    if !discovered.is_empty() {
        let added = add_discovered_projects(db, &discovered)?;
        println!(
            "  {} {} new project(s) discovered and added",
            "+".green(),
            added
        );
    }

    // 3. Detect and update git origins for all projects
    let projects = list_projects(&db.conn, None)?;
    let mut origins_updated = 0;
    for project in &projects {
        if let Some(origin) = detect_origin(&project.path) {
            // Only update if changed
            if project.git_origin.as_ref() != Some(&origin) {
                update_git_origin(&db.conn, &project.name, Some(&origin))?;
                origins_updated += 1;
            }
        }
    }
    if origins_updated > 0 {
        println!(
            "  {} {} project origin(s) updated",
            "~".blue(),
            origins_updated
        );
    }

    // 4. Remove projects that are missing or not direct children of tracked_directory
    let projects = list_projects(&db.conn, None)?;
    let tracked_canonical = config.tracked_directory().canonicalize().ok();

    let invalid: Vec<_> = projects
        .iter()
        .filter(|p| {
            // Remove if directory doesn't exist
            if !p.path.exists() {
                return true;
            }
            // Remove if not a DIRECT child of tracked_directory (no recursive tracking)
            if let Some(ref tracked) = tracked_canonical {
                if let Ok(project_canonical) = p.path.canonicalize() {
                    // Parent must be exactly tracked_directory
                    if let Some(parent) = project_canonical.parent() {
                        return parent != tracked.as_path();
                    }
                    return true; // No parent = invalid
                }
            }
            false
        })
        .collect();

    if !invalid.is_empty() {
        let mut removed = 0;
        for p in &invalid {
            let reason = if !p.path.exists() {
                "missing"
            } else {
                "not direct child of tracked directory"
            };
            if remove_project(&db.conn, &p.name)? {
                println!("  {} {} ({})", "-".red(), p.name, reason);
                removed += 1;
            }
        }
        println!("  {} {} project(s) removed total", "-".red(), removed);
    }

    println!("{}", "Sync complete.".green());
    Ok(())
}
