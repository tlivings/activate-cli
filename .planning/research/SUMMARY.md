# Project Research Summary

**Project:** activate
**Domain:** CLI Project Management/Workspace Tools in Rust
**Researched:** 2026-02-19
**Confidence:** HIGH

## Executive Summary

The research reveals that "activate" should be built as a Rust CLI tool that manages project state lifecycles (active/inactive/archived) with deep shell and Git integration. This represents a unique positioning between simple directory jumpers like zoxide and complex workspace managers like tmuxinator. The tool's core value proposition is combining fast project navigation with state-aware project management, allowing developers to maintain mental context about which projects are currently active versus archived.

The recommended approach uses a synchronous Rust architecture with clap for CLI parsing, rusqlite for local state persistence, and git2 for repository operations. The most critical technical challenges involve proper shell command escaping across platforms, consistent path normalization to avoid duplicates, and Windows-specific database locking issues. These must be addressed in the initial architecture to avoid costly refactoring later.

Key risks center around shell integration complexity and cross-platform compatibility. Mitigation strategies include using established escaping libraries (shell-escape), implementing comprehensive path canonicalization before any database operations, and configuring SQLite with WAL mode and appropriate timeouts for Windows compatibility. The architecture should be designed with clear separation between command handling, data persistence, and shell integration layers to isolate platform-specific complexity.

## Key Findings

### Recommended Stack

The research identifies a mature Rust CLI stack centered on clap v4.5+ for argument parsing, rusqlite for embedded database operations, and git2 for Git integration. This stack is battle-tested across popular Rust CLI tools like ripgrep and bat, providing confidence in production readiness.

**Core technologies:**
- **Rust 1.80+**: Core language — memory safety, cross-platform support, excellent CLI ergonomics
- **clap 4.5.60**: CLI parsing — de facto standard with derive macros and shell completion generation
- **rusqlite 0.38.0**: Database — synchronous SQLite bindings perfect for local state management
- **git2 0.20.4**: Git operations — type-safe libgit2 bindings for cloning and status checks
- **serde/toml**: Configuration — standard serialization with TOML for human-editable configs
- **directories 6.0.0**: Path handling — platform-specific directory resolution (XDG, Windows AppData)

### Expected Features

Analysis of similar tools reveals clear user expectations divided into table stakes and differentiators.

**Must have (table stakes):**
- Quick navigation to project directories — users expect instant project switching
- Fuzzy/partial name matching — modern CLI standard following fzf patterns
- Shell integration with cd on activate — seamless shell experience required
- Recent/frecency-based sorting — smart defaults based on usage patterns
- Project listing with filtering — basic discovery and management
- Tab completion — standard CLI ergonomics

**Should have (competitive):**
- Project state management (active/inactive/archived) — unique mental model differentiator
- GitHub integration for cloning and status — reduces context switching
- Project metadata and tagging — organization beyond directories
- Environment auto-loading — project-specific configurations
- Tmux session management — deep workflow integration

**Defer (v2+):**
- Cross-machine synchronization — complex implementation, unclear demand
- Task/todo integration — scope expansion beyond core value
- Project health checks — nice to have but not essential

### Architecture Approach

The research points to a layered architecture with clear separation of concerns: CLI parsing layer, business logic layer with command handlers, and data access layer for database and filesystem operations. This follows patterns established by successful Rust CLI tools.

**Major components:**
1. **CLI Parser** — clap-based command routing with type-safe argument handling
2. **Command Handlers** — isolated modules for activate, list, sync operations
3. **Database Layer** — rusqlite with migrations, typed queries, and proper transaction handling
4. **Filesystem Scanner** — project discovery with gitignore-style exclusions
5. **Shell Generator** — platform-specific shell integration scripts
6. **Output Formatter** — pluggable formatters for table, JSON, TSV output

### Critical Pitfalls

The research identifies several domain-specific pitfalls that commonly affect CLI tools in this space:

1. **Shell integration string escaping** — Improper escaping causes command injection or silent failures. Use shell-escape crate and test with special characters.
2. **Database file locking on Windows** — Different locking semantics cause "database locked" errors. Configure SQLite with WAL mode and 5+ second timeout.
3. **Inconsistent path normalization** — Same project gets multiple entries due to path variations. Always canonicalize paths before database operations.
4. **Blocking filesystem operations** — Scanning large directories freezes the CLI. Implement progress indicators and consider async for network mounts.
5. **Git integration assumptions** — Assuming standard git setups breaks for worktrees and custom configs. Use git2 library and handle edge cases.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation & Core Data Model
**Rationale:** Establishes error handling patterns, configuration structure, and database schema that all other features depend on. Path normalization must be correct from the start to avoid data corruption.
**Delivers:** Basic project CRUD operations, database with migrations, configuration system
**Addresses:** Add/remove projects, project listing, config file support (from FEATURES.md)
**Avoids:** Path normalization bugs, panic-in-production pitfall through proper error handling setup

### Phase 2: State Management & Navigation
**Rationale:** Implements the core differentiator (state management) and primary value prop (quick navigation) before adding complexity
**Delivers:** Project state tracking, fuzzy matching, frecency sorting
**Uses:** rusqlite for state persistence, fuzzy matching algorithms
**Implements:** Command handlers for activate/deactivate, state transition rules

### Phase 3: Shell Integration
**Rationale:** Critical for seamless UX but complex cross-platform work. Needs solid foundation from Phase 1-2.
**Delivers:** Shell functions for bash/zsh/fish/PowerShell, tab completions
**Addresses:** Shell integration with cd, tab completion (from FEATURES.md)
**Avoids:** Shell escaping vulnerabilities through proper library usage

### Phase 4: Git & GitHub Integration
**Rationale:** High-value differentiator that builds on established shell integration
**Delivers:** Clone projects from GitHub, check git status, detect repository URLs
**Uses:** git2 library for operations
**Avoids:** Git assumption pitfalls by handling worktrees and non-standard configs

### Phase 5: Enhanced Features
**Rationale:** Value-add features that enhance the core experience once foundation is solid
**Delivers:** Project metadata/tags, environment auto-loading, basic tmux integration
**Implements:** Extended database schema for metadata

### Phase 6: Performance & Polish
**Rationale:** Optimization and UX improvements based on real usage patterns
**Delivers:** Async filesystem operations, progress indicators, batch operations
**Addresses:** Activity tracking, smart suggestions

### Phase Ordering Rationale

- Foundation must come first to establish patterns that prevent panics and data corruption
- State management before shell integration ensures core value prop works even without shell hooks
- Shell integration before Git operations provides the integration layer needed for repository management
- Enhanced features only after core workflow is validated and stable
- Performance optimization last when actual bottlenecks are identified

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3 (Shell Integration):** Complex cross-platform concerns, each shell has unique syntax
- **Phase 4 (Git Integration):** Wide variety of git configurations and edge cases to handle
- **Phase 5 (Enhanced Features):** Tmux integration varies by version and configuration

Phases with standard patterns (skip research-phase):
- **Phase 1 (Foundation):** Well-established Rust CLI patterns
- **Phase 2 (State Management):** Simple CRUD with state machine logic
- **Phase 6 (Performance):** Standard optimization techniques

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Verified against official docs, proven in production tools |
| Features | HIGH | Based on analysis of multiple similar tools and user expectations |
| Architecture | HIGH | Follows established Rust CLI patterns from successful projects |
| Pitfalls | HIGH | Specific examples from real tool development experiences |

**Overall confidence:** HIGH

### Gaps to Address

While research confidence is high, some areas need validation during implementation:

- **Tmux integration complexity:** Different tmux versions may require adaptation - validate with tmux 2.x and 3.x
- **Windows shell support:** PowerShell integration less documented than Unix shells - may need iteration
- **Performance at scale:** Exact limits for project count before optimization needed - benchmark during development

## Sources

### Primary (HIGH confidence)
- Rust CLI Book — official patterns and best practices
- clap, rusqlite, git2 official documentation — verified versions and capabilities
- ripgrep, bat, exa codebases — architecture patterns from successful tools

### Secondary (MEDIUM confidence)
- GitHub: tmuxinator, zoxide, direnv — feature analysis and user expectations
- SQLite documentation — locking behavior and WAL mode configuration
- Shell escaping security advisories — common vulnerability patterns

### Tertiary (LOW confidence)
- Community forums on tmux integration — implementation details may vary
- Windows-specific CLI challenges — based on issue tracker patterns

---
*Research completed: 2026-02-19*
*Ready for roadmap: yes*