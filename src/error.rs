use anyhow::Result;
use std::fmt;

/// Type alias for our Result type
pub type ActivateResult<T> = Result<T>;

/// Custom error types for activate
#[derive(Debug)]
pub enum ActivateError {
    ProjectNotFound(String),
    ProjectAlreadyExists(String),
    InvalidPath(String),
    DatabaseError(String),
}

impl fmt::Display for ActivateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivateError::ProjectNotFound(name) => {
                write!(f, "Project '{}' not found in database", name)
            }
            ActivateError::ProjectAlreadyExists(name) => {
                write!(f, "Project '{}' already exists in database", name)
            }
            ActivateError::InvalidPath(path) => {
                write!(f, "Invalid path: {}", path)
            }
            ActivateError::DatabaseError(msg) => {
                write!(f, "Database error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ActivateError {}