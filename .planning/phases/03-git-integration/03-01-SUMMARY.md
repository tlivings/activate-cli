---
phase: 03-git-integration
plan: 01
subsystem: cli
tags: [clap, shell-integration, bash, zsh, fish]

# Dependency graph
requires:
  - phase: 02-state-management-navigation
    provides: navigation, TUI, query command, shell wrappers
provides:
  - Flag-based CLI structure where positional arg always means project name
  - Updated shell wrappers using --query and --init flags
  - Add and remove commands exposed in CLI
affects: [shell-integration, user-experience, project-navigation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Flag-based CLI instead of subcommands for namespace clarity
    - Positional argument reserved for primary action (navigation)

key-files:
  created: []
  modified:
    - src/cli.rs
    - src/main.rs
    - src/commands/mod.rs
    - src/shell/wrapper.rs

key-decisions:
  - "Positional argument always means project name to navigate to"
  - "All management operations are flags, not subcommands"
  - "Short flags for common ops: -l list, -a add, -r remove, -s sync, -d deactivate, -q query"

patterns-established:
  - "Flag dispatch order: explicit flags first, then positional, then TUI"
  - "Shell wrappers check for flags in case statement before query+cd"

# Metrics
duration: 8min
completed: 2026-02-20
---

# Phase 3 Plan 1: CLI Refactor Summary

**Refactored CLI from subcommands to flags so positional argument always means project navigation**

## Performance

- **Duration:** 8 min 16s
- **Started:** 2026-02-20T14:45:03Z
- **Completed:** 2026-02-20T14:53:19Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- CLI restructured to use flags instead of subcommands
- Positional argument now exclusively means project name to navigate to
- Shell wrappers (bash, zsh, fish) updated for new flag-based syntax
- Add and remove commands properly exposed in commands module

## Task Commits

Each task was committed atomically:

1. **Task 1: Restructure CLI from subcommands to flags** - `69b2c66` (refactor)
2. **Task 2: Update main.rs dispatcher for flag-based CLI** - `646c41a` (feat)
3. **Task 3: Update shell wrapper functions for flag-based CLI** - `5a7674e` (feat)

## Files Created/Modified
- `src/cli.rs` - Flag-based Cli struct replacing Commands enum
- `src/main.rs` - Dispatcher using if/else chain for flag checks
- `src/commands/mod.rs` - Added pub mod add and pub mod remove
- `src/shell/wrapper.rs` - Updated all shell wrappers for flag syntax

## Decisions Made
- Kept original error handling pattern using anyhow's chain() method
- Added add and remove modules to commands/mod.rs (they existed but weren't exported)
- Used same function signatures as existing commands (e.g., execute_add takes name_override parameter)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing module exports in commands/mod.rs**
- **Found during:** Task 2
- **Issue:** add.rs and remove.rs existed but weren't exported from commands/mod.rs
- **Fix:** Added `pub mod add` and `pub mod remove` to commands/mod.rs
- **Files modified:** src/commands/mod.rs
- **Verification:** cargo build succeeds
- **Committed in:** 646c41a (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Essential fix for CLI to compile. No scope creep.

## Issues Encountered
None - plan executed with one minor deviation (missing module exports).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- CLI foundation complete for Phase 3 git integration
- --add, --remove, --list, --sync all accessible via flags
- Shell wrappers ready for new operations

---
*Phase: 03-git-integration*
*Completed: 2026-02-20*

## Self-Check: PASSED

All files verified:
- src/cli.rs
- src/main.rs
- src/commands/mod.rs
- src/shell/wrapper.rs

All commits verified:
- 69b2c66
- 646c41a
- 5a7674e
