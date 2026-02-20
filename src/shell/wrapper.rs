use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
        }
    }
}

impl std::str::FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => Err(anyhow::anyhow!(
                "Unsupported shell: {}. Supported: bash, zsh, fish",
                s
            )),
        }
    }
}

/// Get the path to the current executable for embedding in shell wrappers
fn get_exe_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "activate".to_string())
}

pub fn generate_wrapper(shell: Shell, func_name: &str) -> String {
    let exe = get_exe_path();
    match shell {
        Shell::Bash => generate_bash(&exe, func_name),
        Shell::Zsh => generate_zsh(&exe, func_name),
        Shell::Fish => generate_fish(&exe, func_name),
    }
}

fn generate_bash(exe: &str, func_name: &str) -> String {
    format!(
        r#"# Activate shell integration for bash
__ACTIVATE_BIN="{exe}"

{func_name}() {{
    local result
    result="$("$__ACTIVATE_BIN" "$@")"
    [[ -n "$result" && -d "$result" ]] && builtin cd -- "$result"
}}

# Tab completion
_{func_name}_completions() {{
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    if [[ ${{COMP_CWORD}} -eq 1 ]]; then
        local projects
        projects="$("$__ACTIVATE_BIN" --completions bash --current "$cur" 2>/dev/null)"
        COMPREPLY=($projects)
    fi
}}
complete -F _{func_name}_completions {func_name}
"#,
        exe = exe,
        func_name = func_name
    )
}

fn generate_zsh(exe: &str, func_name: &str) -> String {
    format!(
        r#"# Activate shell integration for zsh
__ACTIVATE_BIN="{exe}"

# Precmd hook for cd (workaround for Warp terminal slowness)
__activate_target=""
__activate_precmd() {{
    if [[ -n "$__activate_target" ]]; then
        cd "$__activate_target"
        __activate_target=""
    fi
}}
precmd_functions+=(__activate_precmd)

{func_name}() {{
    local result
    result="$("$__ACTIVATE_BIN" "$@")"
    [[ -n "$result" && -d "$result" ]] && __activate_target="$result"
}}

# Debug version: skips DB updates to isolate latency
{func_name}_debug() {{
    local result
    result="$("$__ACTIVATE_BIN" --debug-fast "$@")"
    [[ -n "$result" && -d "$result" ]] && __activate_target="$result"
}}

# Tab completion
_{func_name}() {{
    if (( CURRENT == 2 )); then
        local projects=(${{(f)"$("$__ACTIVATE_BIN" --completions zsh 2>/dev/null)"}})
        compadd -a projects
    fi
}}
compdef _{func_name} {func_name}
"#,
        exe = exe,
        func_name = func_name
    )
}

fn generate_fish(exe: &str, func_name: &str) -> String {
    format!(
        r#"# Activate shell integration for fish
set -g __ACTIVATE_BIN "{exe}"

function {func_name}
    set -l result ($__ACTIVATE_BIN $argv)
    if test -n "$result"; and test -d "$result"
        cd $result
    end
end

# Tab completion
complete -c {func_name} -f
complete -c {func_name} -a "($__ACTIVATE_BIN --completions fish 2>/dev/null)"
"#,
        exe = exe,
        func_name = func_name
    )
}
