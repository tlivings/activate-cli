use anyhow::Result;

use crate::database::Database;
use crate::shell::completions::generate_completions;

pub fn execute_completions(
    db: &Database,
    _shell: &str, // Future: shell-specific formatting
    current: Option<&str>,
) -> Result<()> {
    let names = generate_completions(db, current)?;
    for name in names {
        println!("{}", name);
    }
    Ok(())
}
