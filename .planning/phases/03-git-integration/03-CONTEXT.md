# Phase 3: Git Integration - Context

**Gathered:** 2026-02-20
**Status:** Ready for planning

<domain>
## Phase Boundary

1. **CLI Refactor** - Change from subcommands to flags so positional args are always project names
2. **Git Integration** - Clone GitHub repos via URL, track git origins, warn about uncommitted changes
3. **TUI Enhancements** - Add deactivate/archive actions and help overlay
4. **Documentation** - Create README.md

</domain>

<decisions>
## Implementation Decisions

### CLI Structure Refactor
- **Primary action is navigation** - `activate <name>` always means "go to project"
- Subcommands become flags to avoid collision with project names
- Flag mapping:
  - `activate --list` or `-l` (was: `activate list`)
  - `activate --add <path>` or `-a <path>` (was: `activate add`)
  - `activate --remove <name>` or `-r <name>` (was: `activate remove`)
  - `activate --sync` or `-s` (was: `activate sync`)
  - `activate --status <name>` (was: `activate status`)
  - `activate --deactivate <name>` or `-d <name>` (was: `activate deactivate`)
  - `activate --archive <name>` (was: `activate archive`)
  - `activate --init <shell>` (was: `activate init`)
  - `activate --query <name>` or `-q <name>` (was: `activate query`)
- No args = TUI (unchanged)
- Positional arg = project name to navigate to
- TUI Enter key = cd to selected project (outputs path for shell wrapper)

### URL & Cloning Behavior
- Accept HTTPS and SSH URL formats (no shorthand like user/repo)
- Clone destination: always tracked_directory/<repo-name>
- Folder naming: repo name only, no user/ prefix nesting
- Name conflict: error and abort, suggest `activate <name>` if folder exists

### Uncommitted Changes Handling
- Warn on deactivate and archive only (not on activate)
- Non-blocking warning: show message but proceed anyway
- Scope: staged changes, unstaged changes, AND unpushed commits
- Warning format: summary counts only ("3 uncommitted changes, 2 unpushed commits")

### Missing Project Cleanup
- No "missing" state — if folder doesn't exist, auto-remove from database
- Detection happens during list, activate, and sync operations
- No reclone functionality — user deleted it, their problem
- Simplifies GIT-06, GIT-07, GIT-08 requirements into straightforward cleanup

### Git Info Display
- Origin column in list: full URL always (https://github.com/user/repo)
- Non-git projects show "local" in origin column
- Origin detection: on add, activate, and sync (keep fresh)
- Default list is pipe-friendly: no status indicators
- --verbose flag shows git status (dirty, ahead/behind)
- --json output includes git status data

### Claude's Discretion
- Exact error message wording
- How to handle git command failures
- Performance optimization for git status checks

</decisions>

<specifics>
## Specific Ideas

- "List by default is for piping to other actions" — keep output minimal
- User deleted folder = their problem, just clean up the database
- No second-guessing what happened to missing folders

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-git-integration*
*Context gathered: 2026-02-20*
