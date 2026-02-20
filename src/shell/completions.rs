use crate::database::operations::list_projects;
use crate::database::Database;
use anyhow::Result;

/// Generate project names for shell completion
pub fn generate_completions(db: &Database, current: Option<&str>) -> Result<Vec<String>> {
    let projects = list_projects(&db.conn, None)?;

    let names: Vec<String> = projects
        .iter()
        .map(|p| p.name.clone())
        .filter(|name| {
            // Filter by prefix if current word provided
            current.is_none_or(|c| name.to_lowercase().starts_with(&c.to_lowercase()))
        })
        .collect();

    Ok(names)
}
