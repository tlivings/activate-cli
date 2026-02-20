pub mod formatters;

use serde::{Deserialize, Serialize};

// OutputFormat enum for different output types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Table,
    Json,
    Tsv,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Table
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "tsv" => Ok(OutputFormat::Tsv),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}