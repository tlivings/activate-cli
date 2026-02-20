# Phase 3: Git Integration - Context

**Gathered:** 2026-02-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Clone GitHub repos via URL and track git origin for existing projects. Warn about uncommitted changes on state transitions. Auto-cleanup projects whose folders no longer exist.

</domain>

<decisions>
## Implementation Decisions

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
