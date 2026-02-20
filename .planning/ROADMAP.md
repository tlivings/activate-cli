# Roadmap: Activate

**Project:** activate
**Created:** 2026-02-19
**Depth:** quick
**Coverage:** 36/36 requirements mapped

## Overview

This roadmap delivers the `activate` CLI tool in 3 focused phases. Phase 1 establishes the foundation with SQLite storage and basic project operations. Phase 2 implements the core value proposition of quick navigation with state management. Phase 3 adds Git integration for GitHub cloning and repository awareness.

## Phases

### Phase 1: Foundation & Core CRUD

**Goal:** Users can track and list projects in a SQLite database

**Dependencies:** None

**Status:** Complete

**Plans:** 4 plans

Plans:
- [x] 01-01-PLAN.md — Set up Rust project structure and configuration system
- [x] 01-02-PLAN.md — Implement SQLite database layer with schema and migrations
- [x] 01-03-PLAN.md — Implement add and remove commands
- [x] 01-04-PLAN.md — Implement list command with multiple output formats

**Requirements:**
- CFG-01: Config file stored in ~/.config/activate/ directory
- CFG-02: User can configure tracked directory path via config file
- CFG-03: User can add directories to ignore list to exclude from tracking
- CFG-04: SQLite database stores all project metadata
- CFG-05: Database located in ~/.config/activate/db.sqlite
- PROJ-01: User can add project to database by path
- PROJ-02: User can remove project from database
- PROJ-03: User can list all tracked projects
- PROJ-10: Projects store metadata: name, path, state, last touched, git origin

**Success Criteria:**
1. User can run `activate add /path/to/project` and see it stored in database
2. User can run `activate list` and see all tracked projects in a table
3. User can run `activate remove project-name` and have it removed from database
4. Configuration file at ~/.config/activate/config.toml controls tracked directory

---

### Phase 2: State Management & Navigation

**Goal:** Users can quickly navigate to projects and manage their lifecycle states

**Dependencies:** Phase 1 (database and config foundation)

**Status:** Complete

**Plans:** 6 plans

Plans:
- [x] 02-01-PLAN.md — Database migration for visit_count and frecency calculation module
- [x] 02-02-PLAN.md — Fuzzy matching module and query command
- [x] 02-03-PLAN.md — State management commands (activate/deactivate/archive/status)
- [x] 02-04-PLAN.md — Shell integration (init, wrapper generation, completions)
- [x] 02-05-PLAN.md — Automation (auto-demotion, sync, discovery)
- [x] 02-06-PLAN.md — TUI interface with ratatui

**Requirements:**
- NAV-01: User can navigate to project by typing project name
- NAV-02: User can use fuzzy/partial matching to find projects
- NAV-03: Projects are sorted by frecency (frequency + recency of use)
- NAV-04: Shell function changes directory when activating project
- NAV-05: Tab completion works for project names in shell
- NAV-06: Activating non-existent project offers to create it (user requirement)
- PROJ-04: User can filter projects by state (--active, --inactive, --archived)
- PROJ-05: User can view detailed status/info for a project
- PROJ-06: User can activate a project (marks as active, updates last touched)
- PROJ-07: User can deactivate a project (marks as inactive)
- PROJ-08: User can archive a project (marks as archived)
- PROJ-09: Tool auto-discovers new subdirectories in tracked folder and adds as inactive
- OUT-01: Default list output shows table with columns: name, path, last touched, origin
- OUT-02: User can output TSV format for piping to unix tools
- OUT-03: User can output JSON format with --json flag
- OUT-04: Output shows project state visually in human-readable format
- OUT-05: Missing projects are clearly indicated in output
- AUTO-01: Active projects auto-demote to inactive after 2 weeks without changes
- AUTO-02: Auto-demotion happens non-blocking during command execution
- AUTO-03: User can run sync command to manually refresh all project states
- AUTO-04: Tool uses filesystem stat to determine last modified time
- AUTO-05: Last touched timestamp updates on project activation

**Success Criteria:**
1. User can type `activate my-proj` and be instantly cd'd to that project directory
2. User can type `activate proj` and fuzzy match to "my-project" if it's the best match
3. User can run `activate list --active` to see only active projects
4. User sees projects sorted by how recently and frequently they've been used
5. Shell tab completion suggests project names when typing `activate <TAB>`

---

### Phase 3: Git Integration + CLI Refactor

**Goal:** Users can clone projects from GitHub, track repository status, and use flag-based CLI

**Dependencies:** Phase 2 (state management for marking cloned projects as active)

**Status:** Planning Complete

**Plans:** 4 plans

Plans:
- [ ] 03-01-PLAN.md — CLI refactor from subcommands to flags (Wave 1)
- [ ] 03-02-PLAN.md — Git module foundation: clone, status, origin detection (Wave 1)
- [ ] 03-03-PLAN.md — Git integration in commands: URL cloning, warnings, list verbose (Wave 2)
- [ ] 03-04-PLAN.md — TUI enhancements and README documentation (Wave 3)

**Requirements:**
- CLI-01: CLI uses flags instead of subcommands (--list, --add, --remove, etc.)
- CLI-02: Positional argument always means project name to navigate to
- GIT-01: User can activate with GitHub URL to clone repo
- GIT-02: Cloned repo is extracted to folder name and marked as active
- GIT-03: Tool auto-detects git origin for all projects and stores in database
- GIT-04: Tool checks for uncommitted changes before deactivating/archiving
- GIT-05: Tool warns (non-blocking) if uncommitted changes exist
- GIT-06: Missing projects auto-removed from database (no reclone)
- GIT-07: --verbose flag shows git status in list output
- TUI-01: TUI deactivate action - 'd' key deactivates selected project
- TUI-02: TUI archive action - 'a' key archives selected project
- TUI-03: TUI help section - '?' key shows keybinding help overlay
- DOC-01: README.md - Simple, clear project documentation with flag-based syntax

**Success Criteria:**
1. User can run `activate https://github.com/user/repo` and have it cloned, added to database, and activated
2. User sees warning when trying to archive project with uncommitted changes (proceeds anyway)
3. User can run `activate --list --verbose` to see git status
4. Projects show git origin URL in list output when available
5. `activate myproject` works (positional arg = navigation, not subcommand)
6. `activate --list` works (flag-based, was `activate list`)

---

## Progress

| Phase | Status | Started | Completed | Progress |
|-------|--------|---------|-----------|----------|
| Phase 1: Foundation & Core CRUD | Complete | 2026-02-19 | 2026-02-20 | 100% |
| Phase 2: State Management & Navigation | Complete | 2026-02-20 | 2026-02-20 | 100% |
| Phase 3: Git Integration + CLI Refactor | Planning Complete | - | - | 0% |

**Overall:** 66% (2/3 phases complete)

---

## Notes

### Phase Compression

Due to "quick" depth setting, phases are aggressively compressed:
- Phase 1 focuses only on essential foundation
- Phase 2 combines all state management, navigation, and automation features
- Phase 3 isolates Git integration as a distinct capability

### Critical Path

The critical path is Phase 1 -> Phase 2 -> Phase 3. Each phase depends on the previous:
- Phase 2 needs the database from Phase 1 to track states
- Phase 3 needs state management from Phase 2 to mark cloned projects as active

### Phase 3 Wave Structure

Phase 3 uses parallel execution where possible:
- **Wave 1 (parallel):** CLI refactor (03-01) and Git module (03-02) run simultaneously
- **Wave 2:** Command wiring (03-03) depends on both Wave 1 plans
- **Wave 3:** TUI + README (03-04) depends on Wave 2

---
*Roadmap created: 2026-02-19*
*Last updated: 2026-02-20*
