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
#   eval "$(activate init bash)"

# Change directory helper
__activate_cd() {
    \builtin cd -- "$@" || return
}

# Main activation function - replaces 'activate' for navigation
activate() {
    local result
    # If no args or subcommand, pass through to binary
    case "$1" in
        add|remove|list|status|deactivate|archive|sync|init|completions|query|help|--help|-h|--version|-V)
            command activate "$@"
            return
            ;;
    esac
    # Navigation: query and cd
    result="$(command activate query --exclude "$(pwd)" -- "$@")" && __activate_cd "${result}"
}

# Tab completion
_activate_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Complete subcommands
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=($(compgen -W "add remove list status deactivate archive sync init" -- "$cur"))
        # Also add project names for direct navigation
        local projects
        projects="$(command activate completions bash --current "$cur" 2>/dev/null)"
        COMPREPLY+=($projects)
        return
    fi

    # Complete project names for relevant commands
    case "$prev" in
        status|deactivate|archive|remove)
            COMPREPLY=($(command activate completions bash --current "$cur" 2>/dev/null))
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
#   eval "$(activate init zsh)"

# Change directory helper
__activate_cd() {
    \builtin cd -- "$@" || return
}

# Main activation function
activate() {
    local result
    case "$1" in
        add|remove|list|status|deactivate|archive|sync|init|completions|query|help|--help|-h|--version|-V)
            command activate "$@"
            return
            ;;
    esac
    result="$(command activate query --exclude "$(pwd)" -- "$@")" && __activate_cd "${result}"
}

# Tab completion
_activate() {
    local -a commands projects
    commands=(
        'add:Add a project to the database'
        'remove:Remove a project from the database'
        'list:List all tracked projects'
        'status:Show project status'
        'deactivate:Mark project as inactive'
        'archive:Archive a project'
        'sync:Sync project states'
        'init:Initialize shell integration'
    )

    if (( CURRENT == 2 )); then
        _describe 'command' commands
        # Also complete project names
        projects=(${(f)"$(command activate completions zsh 2>/dev/null)"})
        compadd -a projects
    else
        case "${words[2]}" in
            status|deactivate|archive|remove)
                projects=(${(f)"$(command activate completions zsh 2>/dev/null)"})
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
#   activate init fish | source

function activate
    set -l cmd $argv[1]
    switch "$cmd"
        case add remove list status deactivate archive sync init completions query help -h --help -V --version
            command activate $argv
        case '*'
            set -l result (command activate query --exclude (pwd) -- $argv)
            and cd $result
    end
end

# Tab completion
complete -c activate -f
complete -c activate -n "__fish_use_subcommand" -a "add" -d "Add a project"
complete -c activate -n "__fish_use_subcommand" -a "remove" -d "Remove a project"
complete -c activate -n "__fish_use_subcommand" -a "list" -d "List projects"
complete -c activate -n "__fish_use_subcommand" -a "status" -d "Show project status"
complete -c activate -n "__fish_use_subcommand" -a "deactivate" -d "Deactivate a project"
complete -c activate -n "__fish_use_subcommand" -a "archive" -d "Archive a project"
complete -c activate -n "__fish_use_subcommand" -a "sync" -d "Sync project states"
complete -c activate -n "__fish_use_subcommand" -a "init" -d "Initialize shell"

# Complete project names for direct navigation and commands
complete -c activate -n "__fish_use_subcommand" -a "(command activate completions fish 2>/dev/null)"
complete -c activate -n "__fish_seen_subcommand_from status deactivate archive remove" -a "(command activate completions fish 2>/dev/null)"
"#
    .to_string()
}
