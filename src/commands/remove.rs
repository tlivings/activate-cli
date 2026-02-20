use anyhow::{Context, Result};
use colored::Colorize;

use crate::database::Database;

/// Execute the remove command to untrack a project
pub fn execute_remove(db: &Database, name: &str) -> Result<()> {
    // Try to remove the project from the database
    match crate::database::operations::remove_project(&db.conn, name) {
        Ok(removed) => {
            if removed {
                println!(
                    "{} Removed project '{}'",
                    "✓".green().bold(),
                    name.cyan()
                );
                Ok(())
            } else {
                // Project not found - try to provide helpful suggestions
                let suggestions = find_similar_projects(db, name)?;

                let mut error_msg = format!("Project '{}' not found", name);

                if !suggestions.is_empty() {
                    error_msg.push_str("\n\nDid you mean one of these?");
                    for suggestion in suggestions.iter().take(3) {
                        error_msg.push_str(&format!("\n  - {}", suggestion.cyan()));
                    }
                }

                Err(anyhow::anyhow!(error_msg))
            }
        }
        Err(e) => Err(e).context("Failed to remove project from database"),
    }
}

/// Find projects with names similar to the given name
fn find_similar_projects(db: &Database, name: &str) -> Result<Vec<String>> {
    // Get all projects
    let all_projects = crate::database::operations::list_projects(&db.conn, None)
        .context("Failed to list projects for suggestions")?;

    let name_lower = name.to_lowercase();
    let mut similarities: Vec<(String, f32)> = Vec::new();

    for project in all_projects {
        let project_lower = project.name.to_lowercase();

        // Calculate simple similarity score
        let score = calculate_similarity(&name_lower, &project_lower);

        if score > 0.3 {  // Threshold for similarity
            similarities.push((project.name, score));
        }
    }

    // Sort by similarity score (highest first)
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    Ok(similarities.into_iter().map(|(name, _)| name).collect())
}

/// Calculate a simple similarity score between two strings
fn calculate_similarity(s1: &str, s2: &str) -> f32 {
    // Check for substring matches
    if s1.contains(s2) || s2.contains(s1) {
        return 0.8;
    }

    // Check for prefix matches
    if s1.starts_with(s2) || s2.starts_with(s1) {
        return 0.7;
    }

    // Calculate Levenshtein-like similarity
    let max_len = s1.len().max(s2.len()) as f32;
    if max_len == 0.0 {
        return 1.0;
    }

    let common_chars = count_common_chars(s1, s2) as f32;
    common_chars / max_len
}

/// Count the number of common characters between two strings
fn count_common_chars(s1: &str, s2: &str) -> usize {
    let mut count = 0;
    let mut s2_chars: Vec<char> = s2.chars().collect();

    for c1 in s1.chars() {
        if let Some(pos) = s2_chars.iter().position(|&c2| c1 == c2) {
            count += 1;
            s2_chars.remove(pos);
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_similarity() {
        assert!(calculate_similarity("test", "test") > 0.9);
        assert!(calculate_similarity("test", "testing") > 0.5);
        assert!(calculate_similarity("project", "proj") > 0.5);
        assert!(calculate_similarity("foo", "bar") < 0.3);
        assert!(calculate_similarity("activate", "active") > 0.5);
    }

    #[test]
    fn test_count_common_chars() {
        assert_eq!(count_common_chars("abc", "abc"), 3);
        assert_eq!(count_common_chars("abc", "bca"), 3);
        assert_eq!(count_common_chars("abc", "def"), 0);
        assert_eq!(count_common_chars("hello", "helo"), 4);
    }
}