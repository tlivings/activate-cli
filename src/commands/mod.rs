pub mod add;
pub mod completions;
pub mod init;
pub mod list;
pub mod query;
pub mod remove;

use anyhow::Result;

/// Type alias for command results
pub type CommandResult = Result<()>;