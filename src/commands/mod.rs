pub mod activate;
pub mod archive;
pub mod completions;
pub mod deactivate;
pub mod init;
pub mod interactive;
pub mod list;
pub mod query;
pub mod status;
pub mod sync;

use anyhow::Result;

/// Type alias for command results
pub type CommandResult = Result<()>;