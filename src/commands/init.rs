use anyhow::Result;

use crate::shell::{generate_wrapper, Shell};

pub fn execute_init(shell: &str, name: Option<&str>) -> Result<()> {
    let shell: Shell = shell.parse()?;
    let func_name = name.unwrap_or("activate");
    let wrapper = generate_wrapper(shell, func_name);
    println!("{}", wrapper);
    Ok(())
}
