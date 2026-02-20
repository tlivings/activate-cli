use anyhow::Result;

use crate::shell::{generate_wrapper, Shell};

pub fn execute_init(shell: &str) -> Result<()> {
    let shell: Shell = shell.parse()?;
    let wrapper = generate_wrapper(shell);
    println!("{}", wrapper);
    Ok(())
}
