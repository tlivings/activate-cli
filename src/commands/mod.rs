pub mod add;
pub mod list;
pub mod remove;

use anyhow::Result;

/// Type alias for command results
pub type CommandResult = Result<()>;