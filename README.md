# activate

Quick access to any tracked project. Type `activate <name>` and you're instantly in that directory, ready to work.

## Features

- **Fuzzy project navigation** - Type partial names to find projects
- **Frecency sorting** - Most used and recently accessed projects appear first
- **Interactive TUI** - Visual project browser with filtering
- **Git integration** - Clone from URLs, track origins, warn about uncommitted changes
- **Shell integration** - Native cd with tab completion

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

## License

MIT
