# Activate

## What This Is

A Rust CLI tool called `activate` that manages development project states (active/inactive/archived) in a single tracked directory. Provides shell integration for quick navigation and GitHub clone support, solving the problem of too many projects cluttering your development folder and forgetting what you're actively working on.

## Core Value

Quick access to any tracked project — type `activate <name>` and you're instantly in that directory, ready to work.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Track projects in a configured directory using SQLite database
- [ ] Store project metadata: name, path, state, last touched, git origin
- [ ] Support three project states: active, inactive, archived
- [ ] State transition commands (activate, deactivate, archive)
- [ ] Shell function integration for directory changes (cd to project on activate)
- [ ] List projects with table format showing name, path, last touched, origin
- [ ] Filter listings by state (--active, --inactive, --archived)
- [ ] Auto-discover new subdirectories and add as inactive
- [ ] Auto-demote active projects to inactive after 2 weeks of no changes (non-blocking)
- [ ] Pipable output: TSV by default, JSON with --json flag
- [ ] GitHub URL support: clone repo, add to database, activate automatically
- [ ] Extract repo name from GitHub URL as folder name
- [ ] Detect and store git origin for all projects with remotes
- [ ] Warn on uncommitted changes when deactivating/archiving (with confirmation)
- [ ] Auto-reclone missing projects when activated (if git origin known)
- [ ] Use filesystem stat to track last modified time
- [ ] Manual sync command to refresh all project states
- [ ] Ignore list for excluding subdirectories from tracking
- [ ] Clean up database entries when folders are removed from tracked directory

### Out of Scope

- Multi-directory tracking — single configured directory only, keeps it simple
- Automatic directory change without shell integration — requires shell function, documented in README
- Real-time filesystem watching — scans on demand to keep tool fast
- Project tagging/categorization — three states sufficient for v1
- Time tracking/analytics — just tracks last touched for auto-inactivation
- Interactive TUI — pure CLI tool, composable with other unix tools

## Context

Development work involves many projects: experiments, side projects, client work, open source contributions. Over time, the development directory fills with dozens or hundreds of folders. It becomes hard to:
- Remember which projects are actively being worked on
- Navigate quickly between projects
- Clean up old/stale work
- Track which projects have uncommitted changes
- Reclone deleted projects when needed

The tool treats project state as metadata, separate from the filesystem, enabling quick filtering, navigation, and lifecycle management. Shell integration makes `activate <name>` feel native, like cd but with project awareness.

## Constraints

- **Binary**: Single executable — no runtime dependencies, easy install
- **Storage**: SQLite database in config directory — structured queries, single file, no server needed
- **Config location**: ~/.config/activate/ — XDG-compliant on Linux/Mac
- **Shell integration**: Manual function install — user sources provided shell function in .bashrc/.zshrc
- **Tracked directory**: Single configured directory — all subdirectories treated as projects
- **Platform**: Cross-platform Rust — works on Mac, Linux, Windows

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Go | Excellent CLI tooling (clap, rusqlite), single binary, fast | — Pending |
| SQLite over JSON | Need to query by state, filter by time, handle missing projects efficiently | — Pending |
| Auto-discovery on scan | Balance between manual tracking (tedious) and full automation (noisy) | — Pending |
| Shell function for cd | CLI tool can't change parent shell's directory, shell function is standard pattern | — Pending |
| TSV default output | Unix philosophy: easy to pipe to cut/awk/grep, add --json for scripts | — Pending |
| Three states only | Active (working now), Inactive (pause/stale), Archived (done/reference) covers most needs | — Pending |
| Single tracked directory | Simplifies mental model, encourages organization, reduces scope | — Pending |
| Auto-reclone on activate | Seamless workflow: if project missing but has origin, just get it back | — Pending |

---
*Last updated: 2026-02-19 after initialization*
