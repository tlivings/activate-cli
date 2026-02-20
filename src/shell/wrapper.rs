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

pub fn generate_wrapper(shell: Shell) -> String {
    let exe = get_exe_path();
    match shell {
        Shell::Bash => generate_bash(&exe),
        Shell::Zsh => generate_zsh(&exe),
        Shell::Fish => generate_fish(&exe),
    }
}

fn generate_bash(exe: &str) -> String {
    format!(
        r#"# Activate shell integration for bash
# Add this to your ~/.bashrc:
#   eval "$({exe} --init bash)"

# Path to activate binary
__ACTIVATE_BIN="{exe}"

# Change directory helper
__activate_cd() {{
    \builtin cd -- "$@" || return
}}

# Main activation function - replaces 'activate' for navigation
activate() {{
    # If no arguments, run interactive mode
    if [[ $# -eq 0 ]]; then
        local result
        result="$("$__ACTIVATE_BIN")"
        [[ -n "$result" ]] && __activate_cd "$result"
        return
    fi

    # Check for flags that don't need cd
    case "$1" in
        -l|--list|-a|--add|-r|--remove|-s|--sync|--status|-d|--deactivate|--archive|--init|-c|--config|-h|--help|-V|--version)
            "$__ACTIVATE_BIN" "$@"
            return
            ;;
    esac

    # Otherwise, it's a project name - query and cd
    local result
    result=$("$__ACTIVATE_BIN" --query "$1" --exclude "$PWD" 2>/dev/null)

    if [[ -n "$result" && -d "$result" ]]; then
        __activate_cd "$result"
    else
        # If query fails, pass through to activate (might be URL or create prompt)
        "$__ACTIVATE_BIN" "$@"
    fi
}}

# Tab completion
_activate_completions() {{
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    local prev="${{COMP_WORDS[COMP_CWORD-1]}}"

    # Complete flags
    if [[ ${{COMP_CWORD}} -eq 1 ]]; then
        COMPREPLY=($(compgen -W "--list --add --remove --sync --status --deactivate --archive --init --config -l -a -r -s -d -c" -- "$cur"))
        # Also add project names for direct navigation
        local projects
        projects="$("$__ACTIVATE_BIN" --completions bash --current "$cur" 2>/dev/null)"
        COMPREPLY+=($projects)
        return
    fi

    # Complete project names for relevant flags
    case "$prev" in
        --status|--deactivate|-d|--archive|--remove|-r)
            COMPREPLY=($("$__ACTIVATE_BIN" --completions bash --current "$cur" 2>/dev/null))
            ;;
    esac
}}
complete -F _activate_completions activate
"#,
        exe = exe
    )
}

fn generate_zsh(exe: &str) -> String {
    format!(
        r#"# Activate shell integration for zsh
# Add this to your ~/.zshrc:
#   eval "$({exe} --init zsh)"

# Path to activate binary
__ACTIVATE_BIN="{exe}"

# Change directory helper
__activate_cd() {{
    \builtin cd -- "$@" || return
}}

# Main activation function
activate() {{
    # If no arguments, run interactive mode
    if [[ $# -eq 0 ]]; then
        local result
        result="$("$__ACTIVATE_BIN")"
        [[ -n "$result" ]] && __activate_cd "$result"
        return
    fi

    # Check for flags that don't need cd
    case "$1" in
        -l|--list|-a|--add|-r|--remove|-s|--sync|--status|-d|--deactivate|--archive|--init|-c|--config|-h|--help|-V|--version)
            "$__ACTIVATE_BIN" "$@"
            return
            ;;
    esac

    # Otherwise, query and cd
    local result
    result=$("$__ACTIVATE_BIN" --query "$1" --exclude "$PWD" 2>/dev/null)

    if [[ -n "$result" && -d "$result" ]]; then
        __activate_cd "$result"
    else
        "$__ACTIVATE_BIN" "$@"
    fi
}}

# Tab completion
_activate() {{
    local -a flags projects
    flags=(
        '--list:List all tracked projects'
        '-l:List all tracked projects'
        '--add:Add a project by path'
        '-a:Add a project by path'
        '--remove:Remove a project by name'
        '-r:Remove a project by name'
        '--sync:Synchronize project states'
        '-s:Synchronize project states'
        '--status:Show project status'
        '--deactivate:Deactivate a project'
        '-d:Deactivate a project'
        '--archive:Archive a project'
        '--init:Initialize shell integration'
        '--config:Open configuration file'
        '-c:Open configuration file'
    )

    if (( CURRENT == 2 )); then
        _describe 'flag' flags
        # Also complete project names
        projects=(${{(f)"$("$__ACTIVATE_BIN" --completions zsh 2>/dev/null)"}})
        compadd -a projects
    else
        case "${{words[2]}}" in
            --status|--deactivate|-d|--archive|--remove|-r)
                projects=(${{(f)"$("$__ACTIVATE_BIN" --completions zsh 2>/dev/null)"}})
                compadd -a projects
                ;;
        esac
    fi
}}
compdef _activate activate
"#,
        exe = exe
    )
}

fn generate_fish(exe: &str) -> String {
    format!(
        r#"# Activate shell integration for fish
# Add this to your ~/.config/fish/config.fish:
#   {exe} --init fish | source

# Path to activate binary
set -g __ACTIVATE_BIN "{exe}"

function activate
    # If no arguments, run interactive mode
    if test (count $argv) -eq 0
        set -l result ($__ACTIVATE_BIN)
        and cd $result
        return
    end

    # Check for flags that don't need cd
    switch $argv[1]
        case -l --list -a --add -r --remove -s --sync --status -d --deactivate --archive --init -c --config -h --help -V --version
            $__ACTIVATE_BIN $argv
            return
    end

    # Otherwise, query and cd
    set -l result ($__ACTIVATE_BIN --query $argv[1] --exclude $PWD 2>/dev/null)

    if test -n "$result"; and test -d "$result"
        cd $result
    else
        $__ACTIVATE_BIN $argv
    end
end

# Tab completion
complete -c activate -f
complete -c activate -n "__fish_use_subcommand" -s l -l list -d "List projects"
complete -c activate -n "__fish_use_subcommand" -s a -l add -d "Add a project"
complete -c activate -n "__fish_use_subcommand" -s r -l remove -d "Remove a project"
complete -c activate -n "__fish_use_subcommand" -s s -l sync -d "Sync project states"
complete -c activate -n "__fish_use_subcommand" -l status -d "Show project status"
complete -c activate -n "__fish_use_subcommand" -s d -l deactivate -d "Deactivate a project"
complete -c activate -n "__fish_use_subcommand" -l archive -d "Archive a project"
complete -c activate -n "__fish_use_subcommand" -l init -d "Initialize shell"
complete -c activate -n "__fish_use_subcommand" -s c -l config -d "Open config file"

# Complete project names for direct navigation and commands
complete -c activate -n "__fish_use_subcommand" -a "($__ACTIVATE_BIN --completions fish 2>/dev/null)"
complete -c activate -n "__fish_seen_subcommand_from --status --deactivate -d --archive --remove -r" -a "($__ACTIVATE_BIN --completions fish 2>/dev/null)"
"#,
        exe = exe
    )
}
