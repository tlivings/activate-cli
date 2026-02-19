# Requirements: Activate

**Defined:** 2026-02-19
**Core Value:** Quick access to any tracked project — type `activate <name>` and you're instantly in that directory, ready to work.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Navigation

- [ ] **NAV-01**: User can navigate to project by typing project name
- [ ] **NAV-02**: User can use fuzzy/partial matching to find projects (e.g., "proj" matches "my-project")
- [ ] **NAV-03**: Projects are sorted by frecency (frequency + recency of use)
- [ ] **NAV-04**: Shell function changes directory when activating project
- [ ] **NAV-05**: Tab completion works for project names in shell

### Project Management

- [ ] **PROJ-01**: User can add project to database by path
- [ ] **PROJ-02**: User can remove project from database
- [ ] **PROJ-03**: User can list all tracked projects
- [ ] **PROJ-04**: User can filter projects by state (--active, --inactive, --archived)
- [ ] **PROJ-05**: User can view detailed status/info for a project
- [ ] **PROJ-06**: User can activate a project (marks as active, updates last touched)
- [ ] **PROJ-07**: User can deactivate a project (marks as inactive)
- [ ] **PROJ-08**: User can archive a project (marks as archived)
- [ ] **PROJ-09**: Tool auto-discovers new subdirectories in tracked folder and adds as inactive
- [ ] **PROJ-10**: Projects store metadata: name, path, state, last touched, git origin

### Git Integration

- [ ] **GIT-01**: User can activate with GitHub URL to clone repo
- [ ] **GIT-02**: Cloned repo is extracted to folder name and marked as active
- [ ] **GIT-03**: Tool auto-detects git origin for all projects and stores in database
- [ ] **GIT-04**: Tool checks for uncommitted changes before deactivating/archiving
- [ ] **GIT-05**: Tool warns and prompts for confirmation if uncommitted changes exist
- [ ] **GIT-06**: Activating missing project with git origin auto-reclones it
- [ ] **GIT-07**: Projects without folders are marked as "missing" in database
- [ ] **GIT-08**: Missing projects without git origin are auto-removed from database
- [ ] **GIT-09**: User can force remove project from database even if missing

### Configuration

- [ ] **CFG-01**: Config file stored in ~/.config/activate/ directory
- [ ] **CFG-02**: User can configure tracked directory path via config file
- [ ] **CFG-03**: User can add directories to ignore list to exclude from tracking
- [ ] **CFG-04**: SQLite database stores all project metadata
- [ ] **CFG-05**: Database located in ~/.config/activate/db.sqlite

### Output Formatting

- [ ] **OUT-01**: Default list output shows table with columns: name, path, last touched, origin
- [ ] **OUT-02**: User can output TSV format for piping to unix tools
- [ ] **OUT-03**: User can output JSON format with --json flag
- [ ] **OUT-04**: Output shows project state visually in human-readable format
- [ ] **OUT-05**: Missing projects are clearly indicated in output

### Automation

- [ ] **AUTO-01**: Active projects auto-demote to inactive after 2 weeks without changes
- [ ] **AUTO-02**: Auto-demotion happens non-blocking during command execution
- [ ] **AUTO-03**: User can run sync command to manually refresh all project states
- [ ] **AUTO-04**: Tool uses filesystem stat to determine last modified time
- [ ] **AUTO-05**: Last touched timestamp updates on project activation

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Advanced Integration

- **ADV-01**: Tmux session auto-creation/attachment on project activation
- **ADV-02**: Environment auto-loading (.env files) on project activation
- **ADV-03**: Project metadata/tags for custom organization
- **ADV-04**: Activity tracking analytics (time spent, patterns)
- **ADV-05**: Project templates/scaffolding for quick initialization
- **ADV-06**: Batch operations on multiple projects at once
- **ADV-07**: Smart suggestions based on project activity patterns

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Automatic filesystem watching | Performance overhead, battery drain, unnecessary complexity |
| Built-in VCS operations beyond clone/status | Would reinvent git poorly, scope creep |
| Plugin system | Maintenance burden, focus on core features first |
| GUI/TUI as primary interface | Breaks CLI workflow philosophy, optional only |
| Complex query DSL (SQL-like) | Overkill for typical use cases, steep learning curve |
| Social/sharing features | Privacy concerns, local-first tool philosophy |
| AI-powered suggestions | Overcomplicated, simple heuristics sufficient |
| Embedded scripting language | Complexity explosion, use shell scripts instead |
| Real-time notifications | Unnecessary for project management tool |
| Cross-machine sync in v1 | Complex feature, wait for user demand validation |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| NAV-01 | Phase 2 | Pending |
| NAV-02 | Phase 2 | Pending |
| NAV-03 | Phase 2 | Pending |
| NAV-04 | Phase 2 | Pending |
| NAV-05 | Phase 2 | Pending |
| PROJ-01 | Phase 1 | Pending |
| PROJ-02 | Phase 1 | Pending |
| PROJ-03 | Phase 1 | Pending |
| PROJ-04 | Phase 2 | Pending |
| PROJ-05 | Phase 2 | Pending |
| PROJ-06 | Phase 2 | Pending |
| PROJ-07 | Phase 2 | Pending |
| PROJ-08 | Phase 2 | Pending |
| PROJ-09 | Phase 2 | Pending |
| PROJ-10 | Phase 1 | Pending |
| GIT-01 | Phase 3 | Pending |
| GIT-02 | Phase 3 | Pending |
| GIT-03 | Phase 3 | Pending |
| GIT-04 | Phase 3 | Pending |
| GIT-05 | Phase 3 | Pending |
| GIT-06 | Phase 3 | Pending |
| GIT-07 | Phase 3 | Pending |
| GIT-08 | Phase 3 | Pending |
| GIT-09 | Phase 3 | Pending |
| CFG-01 | Phase 1 | Pending |
| CFG-02 | Phase 1 | Pending |
| CFG-03 | Phase 1 | Pending |
| CFG-04 | Phase 1 | Pending |
| CFG-05 | Phase 1 | Pending |
| OUT-01 | Phase 2 | Pending |
| OUT-02 | Phase 2 | Pending |
| OUT-03 | Phase 2 | Pending |
| OUT-04 | Phase 2 | Pending |
| OUT-05 | Phase 2 | Pending |
| AUTO-01 | Phase 2 | Pending |
| AUTO-02 | Phase 2 | Pending |
| AUTO-03 | Phase 2 | Pending |
| AUTO-04 | Phase 2 | Pending |
| AUTO-05 | Phase 2 | Pending |

**Coverage:**
- v1 requirements: 36 total
- Mapped to phases: 36
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-19*
*Last updated: 2026-02-19 after roadmap creation*