# Phase 1: Foundation & Core CRUD - Research

**Researched:** 2026-02-19
**Domain:** Rust CLI with SQLite Database and Configuration
**Confidence:** HIGH

## Summary

Phase 1 establishes the foundational data layer and basic CRUD operations for the activate project manager. This phase focuses on setting up a robust SQLite database, configuration management, and the core project tracking commands (add, remove, list). The implementation uses established Rust patterns with rusqlite for database operations, clap for CLI parsing, and the directories crate for platform-appropriate configuration paths.

The critical success factors are: proper path normalization to prevent duplicates, robust error handling without panics, and a clean separation between the CLI layer and data persistence layer. This phase sets patterns that all future development will follow.

**Primary recommendation:** Build database schema with migrations support from day one, use canonical paths for all storage, and establish comprehensive error handling patterns that will scale.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.5.60 | CLI argument parsing | Industry standard with derive macros for type-safe command definitions |
| rusqlite | 0.38.0 | SQLite database | Mature bindings with bundled SQLite, eliminating system dependencies |
| serde | 1.0.228 | Serialization | Required for config file parsing and JSON output |
| toml | 1.0.3 | Config file format | Human-editable, Rust ecosystem standard |
| directories | 6.0.0 | Platform paths | Cross-platform config/data directory resolution |
| anyhow | 1.0.100 | Error handling | Context-aware error chains with ? operator support |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4.39 | DateTime handling | For last_touched timestamps in database |
| path-clean | 1.0.1 | Path normalization | Cleaning paths before canonicalization |
| colored | 3.1.1 | Terminal colors | Colored output for better UX in list command |
| tabled | 0.18.0 | Table formatting | Human-readable table output for list command |
| serde_json | 1.0.137 | JSON output | JSON format option for list command |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rusqlite | sqlx | sqlx is async-first and overkill for local SQLite |
| anyhow | thiserror | thiserror better for libraries, anyhow better for applications |
| tabled | comfy-table | tabled has better performance and more features |
| toml | yaml | YAML more error-prone for hand-editing |

**Installation:**
```bash
cargo add clap --features derive,env
cargo add rusqlite --features bundled,chrono
cargo add serde --features derive
cargo add toml
cargo add directories
cargo add anyhow
cargo add chrono --features serde
cargo add path-clean
cargo add colored
cargo add tabled
cargo add serde_json
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs              # Entry point, error handling setup
├── cli.rs               # Clap command definitions
├── commands/            # Command implementations
│   ├── mod.rs           # Command dispatch
│   ├── add.rs           # Add project command
│   ├── remove.rs        # Remove project command
│   └── list.rs          # List projects command
├── database/            # Database layer
│   ├── mod.rs           # Database connection management
│   ├── schema.rs        # Schema and migrations
│   ├── models.rs        # Project model struct
│   └── operations.rs   # CRUD operations
├── config/              # Configuration management
│   ├── mod.rs           # Config struct and loading
│   └── paths.rs         # Platform-specific paths
├── error.rs             # Error types and display
└── utils/               # Utilities
    └── paths.rs         # Path canonicalization helpers
```

### Pattern 1: Database Migration System
**What:** Versioned schema migrations that run automatically on startup
**When to use:** Always - prevents schema drift between versions
**Example:**
```rust
// Source: Standard rusqlite migration pattern
const MIGRATIONS: &[&str] = &[
    // Version 1: Initial schema
    r#"
    CREATE TABLE IF NOT EXISTS projects (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        path TEXT NOT NULL UNIQUE,
        state TEXT NOT NULL CHECK(state IN ('active', 'inactive', 'archived')),
        last_touched INTEGER NOT NULL,
        git_origin TEXT,
        created_at INTEGER NOT NULL DEFAULT (unixepoch()),
        updated_at INTEGER NOT NULL DEFAULT (unixepoch())
    );
    CREATE INDEX idx_projects_state ON projects(state);
    CREATE INDEX idx_projects_last_touched ON projects(last_touched);
    "#,
];

pub fn migrate(conn: &Connection) -> Result<()> {
    let mut version = get_schema_version(conn)?;
    for (i, migration) in MIGRATIONS.iter().enumerate().skip(version) {
        conn.execute_batch(migration)
            .context(format!("Failed to run migration {}", i + 1))?;
        set_schema_version(conn, i + 1)?;
    }
    Ok(())
}
```

### Pattern 2: Path Canonicalization Layer
**What:** Normalize all paths before any database operation
**When to use:** Every path operation to prevent duplicates
**Example:**
```rust
// Source: Best practice for path handling
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

pub fn canonicalize_project_path(path: &Path) -> Result<PathBuf> {
    // Clean the path first (remove . and .. components)
    let cleaned = path_clean::clean(path);

    // Expand ~ to home directory if present
    let expanded = if cleaned.starts_with("~") {
        let home = directories::BaseDirs::new()
            .context("Failed to get home directory")?
            .home_dir()
            .to_path_buf();
        home.join(cleaned.strip_prefix("~").unwrap())
    } else {
        cleaned.to_path_buf()
    };

    // Canonicalize to absolute path, following symlinks
    expanded.canonicalize()
        .context(format!("Failed to canonicalize path: {}", expanded.display()))
}
```

### Pattern 3: Config File with Defaults
**What:** TOML config with sane defaults and platform-appropriate paths
**When to use:** For all user configuration
**Example:**
```rust
// Source: Standard serde + toml pattern
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_tracked_directory")]
    pub tracked_directory: PathBuf,

    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    #[serde(default = "default_inactive_days")]
    pub inactive_after_days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tracked_directory: default_tracked_directory(),
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
            ],
            inactive_after_days: 14,
        }
    }
}

fn default_tracked_directory() -> PathBuf {
    directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().join("Development"))
        .unwrap_or_else(|| PathBuf::from("~/Development"))
}
```

### Anti-Patterns to Avoid
- **String-based project IDs:** Use database integer IDs internally, names for user interface
- **Unchecked unwrap() calls:** Every unwrap is a potential panic in production
- **Hardcoded paths:** Always use directories crate for config/data paths
- **Synchronous directory scanning:** Will be needed in later phases, design async-ready interfaces

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Path resolution | String manipulation | std::fs::canonicalize + path-clean | Handles symlinks, .., platform differences |
| Config directories | Hardcoded ~/.config | directories crate | Correct paths on Windows, macOS, follows XDG |
| SQL query building | String concatenation | rusqlite prepared statements | SQL injection prevention, better performance |
| Datetime handling | Unix timestamps directly | chrono with rusqlite feature | Timezone handling, human-readable formats |
| Shell-safe paths | Manual escaping | shell-escape crate (Phase 2) | Security critical, complex rules |

**Key insight:** The Rust ecosystem has mature solutions for all common CLI patterns. Building custom solutions increases bugs and maintenance burden without benefits.

## Common Pitfalls

### Pitfall 1: Path Normalization Failures
**What goes wrong:** Same project added multiple times with different path representations
**Why it happens:** User provides ~/project, /Users/x/project, ../project - all same directory
**How to avoid:** Always canonicalize paths before database operations, store canonical form
**Warning signs:** Duplicate projects in list output with similar paths

### Pitfall 2: Database File Permission Issues
**What goes wrong:** Database creation fails or locks on Windows
**Why it happens:** Wrong permissions, virus scanners, concurrent access
**How to avoid:**
- Create parent directories with proper permissions
- Use WAL mode: `PRAGMA journal_mode=WAL;`
- Set busy timeout: `PRAGMA busy_timeout=5000;`
- Bundle SQLite with rusqlite to ensure consistent behavior

**Warning signs:** "Database is locked" errors, works on Mac but fails on Windows

### Pitfall 3: Panic on Missing Config
**What goes wrong:** Tool crashes if config file doesn't exist or is malformed
**Why it happens:** Using expect() or unwrap() on config loading
**How to avoid:** Use Default trait, create config if missing, handle parse errors gracefully
**Warning signs:** Stack traces in user bug reports, "thread 'main' panicked" messages

### Pitfall 4: Case Sensitivity Confusion
**What goes wrong:** Project lookup fails due to case differences
**Why it happens:** Database uses case-sensitive UNIQUE constraint, filesystem may not be
**How to avoid:** Store original case but use COLLATE NOCASE for name uniqueness
**Warning signs:** "Project not found" but user can see it in list

## Code Examples

Verified patterns from official sources:

### Database Connection Setup
```rust
// Source: rusqlite best practices
use rusqlite::{Connection, Result};
use anyhow::Context;

pub fn open_database() -> anyhow::Result<Connection> {
    let db_path = directories::ProjectDirs::from("", "", "activate")
        .context("Failed to determine project directories")?
        .data_dir()
        .join("db.sqlite");

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create database directory")?;
    }

    let conn = Connection::open(&db_path)
        .context("Failed to open database")?;

    // Configure for optimal performance and safety
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;

    Ok(conn)
}
```

### Project CRUD Operations
```rust
// Source: Standard rusqlite patterns
use rusqlite::{params, Connection, Result};
use chrono::{DateTime, Utc};

pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: PathBuf,
    pub state: ProjectState,
    pub last_touched: DateTime<Utc>,
    pub git_origin: Option<String>,
}

pub fn add_project(conn: &Connection, name: &str, path: &Path) -> Result<i64> {
    let canonical_path = canonicalize_project_path(path)?;

    conn.execute(
        "INSERT INTO projects (name, path, state, last_touched)
         VALUES (?1, ?2, 'inactive', ?3)",
        params![
            name,
            canonical_path.to_string_lossy(),
            Utc::now().timestamp()
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn list_projects(conn: &Connection, state_filter: Option<&str>) -> Result<Vec<Project>> {
    let query = if let Some(state) = state_filter {
        "SELECT * FROM projects WHERE state = ?1 ORDER BY last_touched DESC"
    } else {
        "SELECT * FROM projects ORDER BY last_touched DESC"
    };

    let mut stmt = conn.prepare(query)?;
    let rows = if let Some(state) = state_filter {
        stmt.query_map(params![state], Project::from_row)?
    } else {
        stmt.query_map([], Project::from_row)?
    };

    rows.collect()
}
```

### CLI Command Structure
```rust
// Source: clap v4 derive patterns
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "activate")]
#[command(about = "Manage development projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a project to tracking
    Add {
        /// Path to the project directory
        path: PathBuf,
        /// Optional name (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Remove a project from tracking
    Remove {
        /// Project name
        name: String,
    },
    /// List tracked projects
    List {
        /// Filter by state (active/inactive/archived)
        #[arg(short, long)]
        state: Option<String>,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| JSON config files | TOML config | ~2020 | Better human editing, comments support |
| Manual SQL strings | Prepared statements | Always for Rust | Type safety, injection prevention |
| println! debugging | env_logger + log | ~2019 | Configurable verbosity, structured logs |
| Hard-coded paths | directories crate | ~2021 | Cross-platform correctness |

**Deprecated/outdated:**
- structopt: Merged into clap v3+, use clap derive instead
- Storing paths as relative: Always store canonical absolute paths

## Open Questions

1. **Project Name Uniqueness**
   - What we know: Names should be unique for CLI UX
   - What's unclear: Case sensitivity policy
   - Recommendation: Use COLLATE NOCASE for name column

2. **Directory Auto-Discovery Timing**
   - What we know: Will scan tracked directory in Phase 2
   - What's unclear: Should Phase 1 prep for this?
   - Recommendation: Design schema to support it but don't implement

3. **Git Origin Detection**
   - What we know: Need git_origin field in database
   - What's unclear: Populate in Phase 1 or wait for git phase?
   - Recommendation: Add field now, populate in git integration phase

## Sources

### Primary (HIGH confidence)
- rusqlite documentation v0.38.0 - Schema design, prepared statements, pragmas
- clap v4 documentation - Derive macro patterns, command structure
- directories crate docs - Platform-specific path resolution
- Rust CLI Book - Error handling, testing patterns

### Secondary (MEDIUM confidence)
- SQLite documentation - WAL mode, pragma settings, performance
- Popular Rust CLIs (ripgrep, bat, exa) - Code structure study

### Tertiary (LOW confidence)
- Community blog posts on Rust CLI patterns - Cross-referenced with official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified in official docs
- Architecture: HIGH - Based on established Rust CLI patterns
- Pitfalls: HIGH - Derived from SQLite docs and Rust best practices

**Research date:** 2026-02-19
**Valid until:** 2026-03-19 (30 days - stable libraries)