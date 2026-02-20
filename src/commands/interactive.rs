use crate::config::open_config_in_editor;
use crate::database::models::ProjectState;
use crate::database::operations::{increment_visit_and_touch, list_projects, update_project_state};
use crate::database::Database;
use crate::navigation::frecency::calculate_frecency;
use anyhow::Result;
use std::io::{self, Write};

pub fn execute_interactive(db: &Database, debug_fast: bool) -> Result<()> {
    // Get projects sorted by frecency
    let mut projects = list_projects(&db.conn, None)?;

    // Sort by frecency (highest first)
    projects.sort_by(|a, b| {
        let fa = calculate_frecency(a.visit_count, a.last_touched);
        let fb = calculate_frecency(b.visit_count, b.last_touched);
        fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
    });

    if projects.is_empty() {
        eprintln!("No projects tracked. Run `activate --sync` to discover projects.");
        return Ok(());
    }

    // Run TUI
    let result =
        crate::tui::run_app(projects, db).map_err(|e| anyhow::anyhow!("TUI error: {}", e))?;

    // If user requested to open config, do that
    if result.open_config {
        return open_config_in_editor();
    }

    // If project selected, activate it and output path
    if let Some(path) = result.selected_path {
        // Skip DB updates in debug_fast mode
        if !debug_fast {
            if let Some(file_name) = path.file_name() {
                if let Some(name) = file_name.to_str() {
                    let _ = update_project_state(&db.conn, name, ProjectState::Active);
                    let _ = increment_visit_and_touch(&db.conn, name);
                }
            }
        }

        // Output path for shell cd (must go to stdout for shell wrapper to capture)
        println!("{}", path.display());
        let _ = io::stdout().flush();
    }

    Ok(())
}
