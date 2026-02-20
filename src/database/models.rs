use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// Project state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectState {
    Active,
    Inactive,
    Archived,
}

impl fmt::Display for ProjectState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectState::Active => write!(f, "active"),
            ProjectState::Inactive => write!(f, "inactive"),
            ProjectState::Archived => write!(f, "archived"),
        }
    }
}

impl FromStr for ProjectState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "active" => Ok(ProjectState::Active),
            "inactive" => Ok(ProjectState::Inactive),
            "archived" => Ok(ProjectState::Archived),
            _ => Err(anyhow::anyhow!("Invalid project state: {}", s)),
        }
    }
}

/// Project model representing a tracked project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: PathBuf,
    pub state: ProjectState,
    pub last_touched: DateTime<Utc>,
    pub git_origin: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Visit count for frecency scoring
    pub visit_count: u32,
}

impl Project {
    /// Create a Project from a database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let state_str: String = row.get("state")?;
        let state_idx = row.as_ref().column_index("state")?;
        let state = ProjectState::from_str(&state_str)
            .map_err(|_| rusqlite::Error::FromSqlConversionFailure(
                state_idx,
                rusqlite::types::Type::Text,
                format!("Invalid project state: {}", state_str).into(),
            ))?;

        let path_str: String = row.get("path")?;
        let path = PathBuf::from(path_str);

        let last_touched_ts: i64 = row.get("last_touched")?;
        let created_at_ts: i64 = row.get("created_at")?;
        let updated_at_ts: i64 = row.get("updated_at")?;

        let last_touched_idx = row.as_ref().column_index("last_touched")?;
        let created_at_idx = row.as_ref().column_index("created_at")?;
        let updated_at_idx = row.as_ref().column_index("updated_at")?;

        Ok(Project {
            id: row.get("id")?,
            name: row.get("name")?,
            path,
            state,
            last_touched: Utc.timestamp_opt(last_touched_ts, 0).single()
                .ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    last_touched_idx,
                    rusqlite::types::Type::Integer,
                    "Invalid timestamp".into(),
                ))?,
            git_origin: row.get("git_origin")?,
            created_at: Utc.timestamp_opt(created_at_ts, 0).single()
                .ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    created_at_idx,
                    rusqlite::types::Type::Integer,
                    "Invalid timestamp".into(),
                ))?,
            updated_at: Utc.timestamp_opt(updated_at_ts, 0).single()
                .ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    updated_at_idx,
                    rusqlite::types::Type::Integer,
                    "Invalid timestamp".into(),
                ))?,
            visit_count: row.get::<_, i64>("visit_count")? as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_state_display() {
        assert_eq!(ProjectState::Active.to_string(), "active");
        assert_eq!(ProjectState::Inactive.to_string(), "inactive");
        assert_eq!(ProjectState::Archived.to_string(), "archived");
    }

    #[test]
    fn test_project_state_from_str() {
        assert_eq!(ProjectState::from_str("active").unwrap(), ProjectState::Active);
        assert_eq!(ProjectState::from_str("ACTIVE").unwrap(), ProjectState::Active);
        assert_eq!(ProjectState::from_str("inactive").unwrap(), ProjectState::Inactive);
        assert_eq!(ProjectState::from_str("archived").unwrap(), ProjectState::Archived);
        assert!(ProjectState::from_str("invalid").is_err());
    }

    #[test]
    fn test_project_state_serde() {
        let state = ProjectState::Active;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"active\"");

        let parsed: ProjectState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ProjectState::Active);
    }
}