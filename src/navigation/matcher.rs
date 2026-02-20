use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::database::models::{Project, ProjectState};
use crate::navigation::frecency::calculate_frecency;

/// Result of matching a project against a query
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    /// Sorted by state first (Active, Inactive, Archived), then by combined score
    pub fn match_projects(&self, query: &str, projects: &[Project]) -> Vec<MatchResult> {
        let mut results: Vec<_> = projects
            .iter()
            .filter_map(|p| {
                self.matcher.fuzzy_match(&p.name, query).map(|fuzzy_score| {
                    let frecency = calculate_frecency(p.visit_count, p.last_touched);
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
            // Sort by state priority first
            let state_order = |s: &ProjectState| match s {
                ProjectState::Active => 1,
                ProjectState::Inactive => 2,
                ProjectState::Archived => 3,
            };

            match state_order(&a.project.state).cmp(&state_order(&b.project.state)) {
                std::cmp::Ordering::Equal => {
                    // Within same state, sort by combined score (higher first)
                    b.combined_score
                        .partial_cmp(&a.combined_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                other => other,
            }
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
    use crate::database::models::ProjectState;
    use chrono::Utc;
    use std::path::PathBuf;

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
            ignored: false,
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
        let projects = vec![create_test_project("my-project", 5)];

        let results = matcher.match_projects("xyz", &projects);
        assert!(results.is_empty());
    }

    #[test]
    fn test_results_sorted_by_state_then_score() {
        let matcher = ProjectMatcher::new();

        // Create projects with different states and visit counts
        let mut proj_active_low = create_test_project("project-active-low", 1);
        proj_active_low.state = ProjectState::Active;

        let mut proj_active_high = create_test_project("project-active-high", 100);
        proj_active_high.state = ProjectState::Active;

        let mut proj_inactive = create_test_project("project-inactive", 200);
        proj_inactive.state = ProjectState::Inactive;

        let mut proj_archived = create_test_project("project-archived", 500);
        proj_archived.state = ProjectState::Archived;

        let projects = vec![
            proj_archived.clone(),
            proj_inactive.clone(),
            proj_active_low.clone(),
            proj_active_high.clone(),
        ];

        let results = matcher.match_projects("proj", &projects);
        assert_eq!(results.len(), 4);

        // Should be sorted by state first (Active, Inactive, Archived)
        // Within Active state, higher visit count first
        assert_eq!(results[0].project.state, ProjectState::Active);
        assert_eq!(results[0].project.name, "project-active-high");
        assert_eq!(results[1].project.state, ProjectState::Active);
        assert_eq!(results[1].project.name, "project-active-low");
        assert_eq!(results[2].project.state, ProjectState::Inactive);
        assert_eq!(results[2].project.name, "project-inactive");
        assert_eq!(results[3].project.state, ProjectState::Archived);
        assert_eq!(results[3].project.name, "project-archived");
    }

    #[test]
    fn test_find_exact_case_insensitive() {
        let matcher = ProjectMatcher::new();
        let projects = vec![create_test_project("MyProject", 5)];

        let result = matcher.find_exact("myproject", &projects);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "MyProject");
    }

    #[test]
    fn test_find_exact_not_found() {
        let matcher = ProjectMatcher::new();
        let projects = vec![create_test_project("my-project", 5)];

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
