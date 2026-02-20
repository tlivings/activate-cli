use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;

use crate::automation::demotion::perform_demotion_check;
use crate::automation::discovery::{add_discovered_projects, discover_new_projects};
use crate::config::Config;
use crate::database::operations::{list_projects, remove_project};
use crate::database::Database;

pub fn execute_sync(db: &Database) -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Syncing project states...".bold());

    // 1. Run demotion check (synchronously for feedback)
    let db_path = db.conn.path().map(|p| PathBuf::from(p));
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

    // 3. Remove missing projects (directories that no longer exist)
    let projects = list_projects(&db.conn, None)?;
    let missing: Vec<_> = projects.iter().filter(|p| !p.path.exists()).collect();

    if !missing.is_empty() {
        let mut removed = 0;
        for p in &missing {
            if remove_project(&db.conn, &p.name)? {
                removed += 1;
            }
        }
        println!(
            "  {} {} project(s) removed (directories no longer exist)",
            "-".red(),
            removed
        );
    }

    println!("{}", "Sync complete.".green());
    Ok(())
}
