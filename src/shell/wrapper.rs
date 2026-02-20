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

pub fn generate_wrapper(shell: Shell) -> String {
    match shell {
        Shell::Bash => generate_bash(),
        Shell::Zsh => generate_zsh(),
        Shell::Fish => generate_fish(),
    }
}

fn generate_bash() -> String {
    r#"# Activate shell integration for bash
# Add this to your ~/.bashrc:
#   eval "$(activate --init bash)"

# Change directory helper
__activate_cd() {
    \builtin cd -- "$@" || return
}

# Main activation function - replaces 'activate' for navigation
activate() {
    # If no arguments, run interactive mode
    if [[ $# -eq 0 ]]; then
        local result
        result="$(command activate)"
        [[ -n "$result" ]] && __activate_cd "$result"
        return
    fi

    # Check for flags that don't need cd
    case "$1" in
        -l|--list|-a|--add|-r|--remove|-s|--sync|--status|-d|--deactivate|--archive|--init|-h|--help|-V|--version)
            command activate "$@"
            return
            ;;
    esac

    # Otherwise, it's a project name - query and cd
    local result
    result=$(command activate --query "$1" --exclude "$PWD" 2>/dev/null)

    if [[ -n "$result" && -d "$result" ]]; then
        __activate_cd "$result"
    else
        # If query fails, pass through to activate (might be URL or create prompt)
        command activate "$@"
    fi
}

# Tab completion
_activate_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Complete flags
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=($(compgen -W "--list --add --remove --sync --status --deactivate --archive --init -l -a -r -s -d" -- "$cur"))
        # Also add project names for direct navigation
        local projects
        projects="$(command activate --completions bash --current "$cur" 2>/dev/null)"
        COMPREPLY+=($projects)
        return
    fi

    # Complete project names for relevant flags
    case "$prev" in
        --status|--deactivate|-d|--archive|--remove|-r)
            COMPREPLY=($(command activate --completions bash --current "$cur" 2>/dev/null))
            ;;
    esac
}
complete -F _activate_completions activate
"#
    .to_string()
}

fn generate_zsh() -> String {
    r#"# Activate shell integration for zsh
# Add this to your ~/.zshrc:
#   eval "$(activate --init zsh)"

# Change directory helper
__activate_cd() {
    \builtin cd -- "$@" || return
}

# Main activation function
activate() {
    # If no arguments, run interactive mode
    if [[ $# -eq 0 ]]; then
        local result
        result="$(command activate)"
        [[ -n "$result" ]] && __activate_cd "$result"
        return
    fi

    # Check for flags that don't need cd
    case "$1" in
        -l|--list|-a|--add|-r|--remove|-s|--sync|--status|-d|--deactivate|--archive|--init|-h|--help|-V|--version)
            command activate "$@"
            return
            ;;
    esac

    # Otherwise, query and cd
    local result
    result=$(command activate --query "$1" --exclude "$PWD" 2>/dev/null)

    if [[ -n "$result" && -d "$result" ]]; then
        __activate_cd "$result"
    else
        command activate "$@"
    fi
}

# Tab completion
_activate() {
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
    )

    if (( CURRENT == 2 )); then
        _describe 'flag' flags
        # Also complete project names
        projects=(${(f)"$(command activate --completions zsh 2>/dev/null)"})
        compadd -a projects
    else
        case "${words[2]}" in
            --status|--deactivate|-d|--archive|--remove|-r)
                projects=(${(f)"$(command activate --completions zsh 2>/dev/null)"})
                compadd -a projects
                ;;
        esac
    fi
}
compdef _activate activate
"#
    .to_string()
}

fn generate_fish() -> String {
    r#"# Activate shell integration for fish
# Add this to your ~/.config/fish/config.fish:
#   activate --init fish | source

function activate
    # If no arguments, run interactive mode
    if test (count $argv) -eq 0
        set -l result (command activate)
        and cd $result
        return
    end

    # Check for flags that don't need cd
    switch $argv[1]
        case -l --list -a --add -r --remove -s --sync --status -d --deactivate --archive --init -h --help -V --version
            command activate $argv
            return
    end

    # Otherwise, query and cd
    set -l result (command activate --query $argv[1] --exclude $PWD 2>/dev/null)

    if test -n "$result"; and test -d "$result"
        cd $result
    else
        command activate $argv
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

# Complete project names for direct navigation and commands
complete -c activate -n "__fish_use_subcommand" -a "(command activate --completions fish 2>/dev/null)"
complete -c activate -n "__fish_seen_subcommand_from --status --deactivate -d --archive --remove -r" -a "(command activate --completions fish 2>/dev/null)"
"#
    .to_string()
}
