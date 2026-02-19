# Architecture Research

**Domain:** Rust CLI Tools
**Researched:** 2026-02-19
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Command Layer (CLI)                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │  Parse  │  │Validate │  │ Config  │  │ Output  │        │
│  │  (clap) │  │  Args   │  │  Load   │  │ Format  │        │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
│       │            │            │            │              │
├───────┴────────────┴────────────┴────────────┴──────────────┤
│                    Business Logic Layer                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐    │
│  │          Command Handlers / Domain Services          │    │
│  │  (State Management, Project Operations, Git Ops)     │    │
│  └─────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                    Data Access Layer                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
│  │ Database │  │Filesystem│  │   Git    │                   │
│  │(rusqlite)│  │   (std)  │  │  (git2)  │                   │
│  └──────────┘  └──────────┘  └──────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| CLI Parser | Parse args, subcommands, flags | clap v4 with derive macros |
| Config Manager | Load/save config, find config dir | directories crate + serde |
| Command Handlers | Execute business logic per command | Module per command pattern |
| Database Layer | SQLite operations, migrations | rusqlite with typed queries |
| Filesystem Scanner | Detect projects, check modifications | std::fs with walkdir |
| Git Integration | Clone repos, check status | git2 crate or shell commands |
| Output Formatter | Human tables, TSV, JSON output | tabled/comfy-table + serde_json |
| Shell Generator | Generate shell integration scripts | Template strings per shell |
| Error Handler | Unified error handling and display | anyhow or thiserror |

## Recommended Project Structure

```
src/
├── main.rs             # Entry point, CLI setup, dispatch
├── cli.rs              # Clap command/arg definitions
├── commands/           # Command handler modules
│   ├── mod.rs          # Command dispatch
│   ├── activate.rs     # Activate command logic
│   ├── list.rs         # List command logic
│   ├── sync.rs         # Sync command logic
│   └── shell.rs        # Shell integration generator
├── database/           # Data persistence layer
│   ├── mod.rs          # Database connection, pool
│   ├── models.rs       # Project struct, state enum
│   ├── queries.rs      # SQL queries, migrations
│   └── migrations.rs   # Schema migration logic
├── scanner/            # Filesystem operations
│   ├── mod.rs          # Scanner trait/interface
│   ├── project.rs      # Project discovery logic
│   └── git.rs          # Git repo detection
├── config/             # Configuration management
│   ├── mod.rs          # Config struct, paths
│   └── defaults.rs     # Default configuration
├── output/             # Output formatting
│   ├── mod.rs          # Output trait
│   ├── table.rs        # Human-readable tables
│   ├── tsv.rs          # TSV formatter
│   └── json.rs         # JSON formatter
├── error.rs            # Error types and handling
└── utils.rs            # Shared utilities
```

### Structure Rationale

- **commands/:** Separation of concerns - each command is isolated, easy to test and modify independently
- **database/:** Encapsulates all SQL and schema management, migrations co-located with queries
- **scanner/:** Abstracts filesystem operations, making it easy to mock for testing
- **output/:** Strategy pattern for different output formats, extensible for new formats
- **Single error.rs:** Centralized error handling promotes consistency across the codebase

## Architectural Patterns

### Pattern 1: Builder Pattern for Commands

**What:** Use builder pattern for complex command configurations
**When to use:** Commands with many optional parameters
**Trade-offs:** More verbose setup, but better ergonomics and validation

**Example:**
```rust
// Using clap's derive API
#[derive(Parser)]
#[command(name = "activate")]
#[command(about = "Manage development projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Make a project active
    Activate {
        /// Project name
        name: String,
        /// Force activation without checks
        #[arg(short, long)]
        force: bool,
    },
    /// List all projects
    List {
        /// Filter by state
        #[arg(short, long)]
        state: Option<ProjectState>,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
}
```

### Pattern 2: Result-based Error Propagation

**What:** Use Result<T, E> throughout with ? operator for error propagation
**When to use:** Always - this is idiomatic Rust
**Trade-offs:** Explicit error handling required, but catches issues at compile time

**Example:**
```rust
use anyhow::{Context, Result};

fn activate_project(name: &str) -> Result<()> {
    let db = open_database()
        .context("Failed to open database")?;

    let project = db.find_project(name)
        .context(format!("Project '{}' not found", name))?;

    if !project.path.exists() && project.git_origin.is_some() {
        clone_repository(&project.git_origin.unwrap())
            .context("Failed to clone repository")?;
    }

    db.update_state(name, ProjectState::Active)
        .context("Failed to update project state")?;

    Ok(())
}
```

### Pattern 3: Type State Pattern for Database Migrations

**What:** Use types to ensure migrations run in correct order
**When to use:** Managing database schema versions
**Trade-offs:** More complex setup, but prevents migration errors at compile time

**Example:**
```rust
pub struct Database {
    conn: Connection,
    version: i32,
}

impl Database {
    pub fn migrate(&mut self) -> Result<()> {
        while self.version < CURRENT_VERSION {
            match self.version {
                0 => self.migrate_v1()?,
                1 => self.migrate_v2()?,
                _ => break,
            }
            self.version += 1;
        }
        Ok(())
    }
}
```

## Data Flow

### Request Flow

```
CLI Input
    ↓
Arg Parser → Command Handler → Service Layer → Database/FS
    ↓              ↓              ↓               ↓
Validation   Business Logic   Operations    Persistence
    ↓              ↓              ↓               ↓
Output ← Format ← Transform ← Query Results ← Storage
```

### State Management

```
Project Discovery (FS Scan)
    ↓
Database Update
    ↓ (query)
State Transition Rules → Update State → Database Write
    ↓
Shell Output / JSON Response
```

### Key Data Flows

1. **Activation Flow:** Parse project name → Check database → Clone if missing → Update state → Generate shell command
2. **List Flow:** Query database by filters → Join with filesystem data → Format output (table/TSV/JSON)
3. **Sync Flow:** Scan tracked directory → Diff with database → Update/insert/delete records → Report changes

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k projects | SQLite with simple queries, full directory scans |
| 1k-10k projects | Add indexes on frequently queried columns, cache git origins |
| 10k+ projects | Consider async I/O for filesystem ops, batch database updates |

### Scaling Priorities

1. **First bottleneck:** Filesystem scanning - add caching layer for modification times
2. **Second bottleneck:** Database queries - add proper indexes, use prepared statements

## Anti-Patterns

### Anti-Pattern 1: Blocking on Network I/O

**What people do:** Clone repositories synchronously in main thread
**Why it's wrong:** Freezes the CLI, poor user experience
**Do this instead:** Show progress indicators, consider async operations for network tasks

### Anti-Pattern 2: Stringly-Typed Everything

**What people do:** Pass strings everywhere, parse repeatedly
**Why it's wrong:** Runtime errors, no compile-time guarantees
**Do this instead:** Use enums for states, newtype pattern for IDs

```rust
// Bad
fn update_state(state: &str) { ... }

// Good
enum ProjectState { Active, Inactive, Archived }
fn update_state(state: ProjectState) { ... }
```

### Anti-Pattern 3: Ignoring Platform Differences

**What people do:** Hardcode paths like ~/.config
**Why it's wrong:** Breaks on Windows, doesn't respect XDG
**Do this instead:** Use directories crate for platform-appropriate paths

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| GitHub | HTTPS clone via git2 or shell | Handle auth via git credentials |
| Shell | Generate function, user sources it | Can't change parent shell directly |
| SQLite | Embedded via rusqlite | Single file, no server needed |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI ↔ Commands | Enum dispatch with data | Type-safe command routing |
| Commands ↔ Database | Repository pattern | Abstract SQL behind methods |
| Scanner ↔ Database | Batch updates | Minimize transaction overhead |
| Commands ↔ Output | Trait-based formatters | Pluggable output formats |

## Build Order Implications

Based on component dependencies, suggested build order:

1. **Foundation** (Week 1)
   - Error types (error.rs)
   - Config structure (config/)
   - Database models (database/models.rs)

2. **Data Layer** (Week 1-2)
   - Database setup and migrations (database/)
   - Basic CRUD operations
   - Filesystem scanner (scanner/)

3. **Core Commands** (Week 2-3)
   - CLI structure (cli.rs, main.rs)
   - Activate command (commands/activate.rs)
   - List command (commands/list.rs)

4. **Integration** (Week 3-4)
   - Git operations (scanner/git.rs)
   - Shell integration (commands/shell.rs)
   - Output formatting (output/)

5. **Polish** (Week 4)
   - Sync command (commands/sync.rs)
   - Auto-inactivation logic
   - Warning systems

This order ensures each layer has its dependencies ready and allows for incremental testing.

## Sources

- Rust CLI Book (official Rust documentation)
- clap v4 documentation and examples
- rusqlite best practices
- Popular Rust CLI tools: ripgrep, fd, exa, bat (architecture study)
- git2-rs documentation

---
*Architecture research for: Rust CLI Tools*
*Researched: 2026-02-19*