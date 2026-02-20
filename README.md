# activate

Quick access to any tracked project. Type `activate <name>` and you're instantly in that directory, ready to work.

## Why?

If you work across multiple projects, you know the pain:

- **Slow context switching** - `cd ~/Development/client-work/project-name`, `cd ../../personal/other-project`, repeat
- **Path fatigue** - Memorizing or searching for exact paths breaks your flow
- **Lost projects** - That side project you started months ago? Good luck finding it
- **Tab completion fails** - Standard shell completion doesn't know which projects matter to you

`activate` solves this by:
- **Learning your habits** - Frequently and recently used projects surface first
- **Working like your brain** - Fuzzy match on partial names, no exact paths needed
- **Staying fast** - No git operations or filesystem scans during navigation
- **Fitting your workflow** - Works with your shell, your terminal, your existing projects

Stop navigating file trees. Start working.

## Features

- **Fuzzy project navigation** - Type partial names to find projects
- **Frecency sorting** - Most used and recently accessed projects appear first
- **Interactive TUI** - Visual project browser with filtering
- **Git integration** - Clone from URLs, track origins
- **Shell integration** - Native cd with tab completion
- **Fast** - No git operations during activation

## Installation

### From source

```bash
git clone https://github.com/YOUR_USERNAME/activate
cd activate
cargo install --path .
```

### First-time setup

On first run, activate will prompt you to set your projects directory:

```bash
$ activate
Welcome to activate!

First-time setup required.

Enter your projects directory (e.g., ~/Development): ~/Projects
Configuration saved to: ~/.config/activate/config.toml
Projects directory: /Users/you/Projects

Run 'activate --sync' to discover existing projects.
```

### Shell setup

Add to your shell config (~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish):

```bash
# bash/zsh
eval "$(activate --init bash)"  # or zsh

# fish
activate --init fish | source
```

### Warp terminal users

Warp has a built-in `activate` command. Use `--name` to create a different function name:

```bash
# Add to ~/.zshrc
eval "$($HOME/.cargo/bin/activate --init zsh --name act)"
```

Then use `act` instead of `activate` for all commands.

## Usage

### Basic navigation

```bash
# Activate a project (fuzzy match)
activate my-proj    # cd to matching project directory

# Interactive mode (no arguments)
activate            # Opens TUI project browser
```

### Project management

```bash
# Add a project
activate --add /path/to/project
activate -a /path/to/project

# Remove a project
activate --remove project-name
activate -r project-name

# List all projects
activate --list
activate -l
activate --list --state active   # Filter by state
activate --list --verbose        # Show git status
```

### Git integration

```bash
# Clone and activate from URL
activate https://github.com/user/repo
activate git@github.com:user/repo.git

# Sync all projects (detect origins, cleanup missing)
activate --sync
activate -s
```

### State management

```bash
# Change project state
activate --deactivate project-name
activate -d project-name
activate --archive project-name

# View project details
activate --status project-name
```

## Configuration

### Edit configuration

```bash
# Open config in your editor
activate --config
activate -c
```

Or in the TUI, press `c` for settings, then `e` to edit.

### Config file

Location: `~/.config/activate/config.toml`

```toml
# Directory containing your projects (required)
tracked_directory = "/Users/you/Development"

# Patterns to ignore during auto-discovery
ignore_patterns = [
    ".git",
    "node_modules",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    ".DS_Store",
]
```

## Interactive TUI

Run `activate` with no arguments to open the interactive browser.

| Key | Action |
|-----|--------|
| j/k or arrows | Navigate |
| Enter | Activate project |
| d | Deactivate project |
| a | Archive project |
| i | Toggle ignore |
| I | Show/hide ignored |
| c | Settings |
| ? | Help |
| Esc | Quit |

## Project states

- **Active** - Currently working on
- **Inactive** - Not recently used (auto-demoted after 14 days)
- **Archived** - Intentionally set aside

## Troubleshooting

### Slow cd in Warp terminal

If changing to certain directories is slow, it's likely Warp's git integration checking repository status. Try:

```bash
# In slow repos, enable git caching
git config core.untrackedCache true
git config core.fsmonitor true
```

### Debug mode

To isolate latency issues, use the debug function (skips database updates):

```bash
act_debug  # Instead of act
```

If `act_debug` is fast but `act` is slow, the issue is in shell/terminal hooks, not activate.

## Development

```bash
# Run tests
cargo test

# Build release
cargo build --release
```

## License

MIT
