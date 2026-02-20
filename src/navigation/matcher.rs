use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::database::models::Project;
use crate::navigation::frecency::calculate_frecency;

/// Result of matching a project against a query
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub project: Project,
    pub fuzzy_score: i64,
    pub frecency: f64,
    pub combined_score: f64,
}

/// Fuzzy matcher for project names with frecency integration
pub struct ProjectMatcher {
    matcher: SkimMatcherV2,
}

impl ProjectMatcher {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Match projects against query, returning sorted results
    /// Combines fuzzy match score with frecency for ranking
    pub fn match_projects(&self, query: &str, projects: &[Project]) -> Vec<MatchResult> {
        let mut results: Vec<_> = projects
            .iter()
            .filter_map(|p| {
                self.matcher.fuzzy_match(&p.name, query).map(|fuzzy_score| {
                    let frecency = calculate_frecency(
                        p.visit_count,
                        p.last_touched,
                    );
                    MatchResult {
                        project: p.clone(),
                        fuzzy_score,
                        frecency,
                        combined_score: fuzzy_score as f64 + frecency,
                    }
                })
            })
            .collect();

        results.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Find exact match by name (case-insensitive)
    pub fn find_exact(&self, name: &str, projects: &[Project]) -> Option<Project> {
        projects
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .cloned()
    }
}

impl Default for ProjectMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;
    use crate::database::models::ProjectState;

    fn create_test_project(name: &str, visit_count: u32) -> Project {
        Project {
            id: 1,
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            state: ProjectState::Active,
            last_touched: Utc::now(),
            visit_count,
            git_origin: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_fuzzy_match_partial_name() {
        let matcher = ProjectMatcher::new();
        let projects = vec![
            create_test_project("my-project", 5),
            create_test_project("other-thing", 3),
        ];

        let results = matcher.match_projects("myproj", &projects);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].project.name, "my-project");
    }

    #[test]
    fn test_fuzzy_match_initials() {
        let matcher = ProjectMatcher::new();
        let projects = vec![
            create_test_project("my-project", 5),
            create_test_project("another-project", 3),
        ];

        let results = matcher.match_projects("mp", &projects);
        assert!(results.iter().any(|r| r.project.name == "my-project"));
    }

    #[test]
    fn test_no_matches_returns_empty() {
        let matcher = ProjectMatcher::new();
        let projects = vec![
            create_test_project("my-project", 5),
        ];

        let results = matcher.match_projects("xyz", &projects);
        assert!(results.is_empty());
    }

    #[test]
    fn test_results_sorted_by_combined_score() {
        let matcher = ProjectMatcher::new();
        let projects = vec![
            create_test_project("project-alpha", 100), // High visit count
            create_test_project("project-beta", 1),    // Low visit count
        ];

        let results = matcher.match_projects("proj", &projects);
        assert_eq!(results.len(), 2);
        // Higher combined score (visit_count) should come first
        assert_eq!(results[0].project.name, "project-alpha");
    }

    #[test]
    fn test_find_exact_case_insensitive() {
        let matcher = ProjectMatcher::new();
        let projects = vec![
            create_test_project("MyProject", 5),
        ];

        let result = matcher.find_exact("myproject", &projects);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "MyProject");
    }

    #[test]
    fn test_find_exact_not_found() {
        let matcher = ProjectMatcher::new();
        let projects = vec![
            create_test_project("my-project", 5),
        ];

        let result = matcher.find_exact("other-project", &projects);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_with_zero_visit_count() {
        let matcher = ProjectMatcher::new();
        let projects = vec![
            create_test_project("my-project", 0), // visit_count is 0
        ];

        let results = matcher.match_projects("myproj", &projects);
        assert_eq!(results.len(), 1);
        // Zero visits should result in zero frecency
        assert!((results[0].frecency - 0.0).abs() < 0.01);
    }
}
