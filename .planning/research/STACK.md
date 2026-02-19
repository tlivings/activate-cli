# Stack Research

**Domain:** CLI Development Tools in Rust
**Researched:** 2026-02-19
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.80+ | Core language | Industry standard for performant CLI tools with memory safety guarantees and excellent cross-platform support |
| clap | 4.5.60 | CLI argument parsing | De facto standard for Rust CLIs with derive macros, excellent help generation, shell completions, and comprehensive validation |
| rusqlite | 0.38.0 | SQLite database access | Mature, synchronous SQLite bindings perfect for local database operations in CLI tools |
| git2 | 0.20.4 | Git operations | Comprehensive libgit2 bindings with type-safe interfaces for cloning, status checks, and repository management |
| serde | 1.0.228 | Serialization framework | Zero-overhead serialization with derive macros, required for config files and data structures |
| toml | 1.0.3 | Configuration parsing | Standard for Rust configuration files, Cargo-compatible, serde integration |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| directories | 6.0.0 | Platform-specific paths | Getting correct config/cache/data directories across Windows/Mac/Linux |
| colored | 3.1.1 | Terminal colors | Adding color to output messages for better UX |
| env_logger | 0.11.9 | Logging framework | Debug logging controlled via RUST_LOG environment variable |
| indicatif | 0.18.4 | Progress bars | Showing progress for long-running operations like scanning directories |
| dialoguer | 0.12.0 | Interactive prompts | User confirmations and interactive selections |
| human-panic | 1.2.3 | Friendly error messages | Converting panics to user-friendly error reports in release builds |
| which | 8.0.0 | Command detection | Finding executables in PATH for shell integration |
| tempfile | 3.17.0 | Temporary files | Safe temporary file handling for shell script generation |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo-watch | Auto-rebuild on changes | Run with `cargo watch -x build` |
| assert_cmd | 2.1.2 | Integration testing framework for CLI commands |
| predicates | 3.1.3 | Test assertions | Works with assert_cmd for output validation |
| criterion | 0.6.0 | Benchmarking | Performance testing for critical paths |

## Installation

```bash
# Initialize project
cargo init activate
cd activate

# Add core dependencies to Cargo.toml
cargo add clap --features derive,env,cargo
cargo add rusqlite --features bundled
cargo add git2
cargo add serde --features derive
cargo add toml
cargo add directories
cargo add colored
cargo add env_logger
cargo add indicatif
cargo add dialoguer
cargo add human-panic
cargo add which
cargo add tempfile

# Add dev dependencies
cargo add --dev assert_cmd
cargo add --dev predicates
cargo add --dev criterion
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| rusqlite | sqlx | When you need async database operations or compile-time SQL verification |
| git2 | gitoxide | When you need pure Rust implementation without C dependencies |
| clap | argh | For minimal binary size in embedded/resource-constrained environments |
| colored | owo-colors | When you need zero-allocation color formatting |
| env_logger | tracing | When building complex async applications with structured logging |
| indicatif | pbr | For simpler progress bar needs with smaller dependency footprint |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| structopt | Deprecated, merged into clap | clap with derive feature |
| diesel (for this project) | Overkill for simple SQLite operations | rusqlite for direct SQL control |
| async runtime (tokio/async-std) | Unnecessary complexity for synchronous CLI tool | Synchronous operations with std library |
| termion | Unix-only, maintenance concerns | crossterm for cross-platform support |
| raw ANSI codes | Platform compatibility issues | colored or crossterm for abstraction |

## Stack Patterns by Variant

**If building a TUI (terminal UI):**
- Add ratatui + crossterm
- Because they provide the most mature cross-platform TUI framework

**If needing async operations:**
- Add tokio runtime + sqlx instead of rusqlite
- Because async is required for concurrent network operations

**If targeting embedded/minimal systems:**
- Use argh instead of clap, skip colored/indicatif
- Because binary size becomes critical constraint

**If building for Windows primarily:**
- Add windows-rs for native Windows API access
- Because some operations require platform-specific APIs

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| clap@4.5.60 | clap_complete@4.5.40 | Use matching major versions for shell completions |
| rusqlite@0.38.0 | bundled feature | Includes SQLite 3.47.2, no system dependency needed |
| serde@1.0.228 | toml@1.0.3 | Both use serde 1.0 traits |
| indicatif@0.18.4 | console@0.16.2 | indicatif depends on console for terminal detection |
| assert_cmd@2.1.2 | predicates@3.1.3 | Designed to work together for testing |

## Platform-Specific Considerations

### Cross-Platform Path Handling
- Use `std::path::PathBuf` for all file paths
- Never hardcode path separators
- Use `directories` crate for standard locations

### Shell Integration
- Generate POSIX-compliant scripts for bash/zsh
- Generate PowerShell scripts for Windows
- Use `which` crate to detect available shells

### Database Location
- Use `directories::ProjectDirs` for database path
- Default: `~/.local/share/activate/activate.db` (Linux/Mac)
- Default: `%APPDATA%\activate\data\activate.db` (Windows)

## Security Considerations

| Concern | Mitigation |
|---------|------------|
| SQL Injection | Use parameterized queries with rusqlite |
| Path Traversal | Validate and sanitize all user-provided paths |
| Git Clone | Validate URLs, consider shallow clones for performance |
| Shell Script Generation | Properly escape all variables in generated scripts |

## Performance Optimizations

| Area | Technique |
|------|-----------|
| Database | Use prepared statements, transaction batching |
| Directory Scanning | Use parallel iteration with rayon if needed |
| Git Operations | Use shallow clones when full history not needed |
| Startup Time | Lazy-load features, avoid unnecessary allocations |

## Sources

- https://docs.rs/clap/latest/clap/ — Verified clap 4.5.60 features and capabilities
- https://docs.rs/rusqlite/latest/rusqlite/ — Confirmed rusqlite 0.38.0 for SQLite operations
- https://docs.rs/git2/latest/git2/ — Checked git2 0.20.4 for Git functionality
- https://docs.rs/serde/latest/serde/ — Verified serde 1.0.228 serialization
- https://docs.rs/toml/latest/toml/ — Confirmed TOML 1.0.3 for configuration
- https://docs.rs/directories/latest/directories/ — Checked directories 6.0.0 for paths
- https://docs.rs/colored/latest/colored/ — Verified colored 3.1.1 for terminal colors
- https://docs.rs/env_logger/latest/env_logger/ — Confirmed env_logger 0.11.9
- https://docs.rs/indicatif/latest/indicatif/ — Checked indicatif 0.18.4 for progress bars
- https://docs.rs/dialoguer/latest/dialoguer/ — Verified dialoguer 0.12.0 for prompts
- https://docs.rs/assert_cmd/latest/assert_cmd/ — Confirmed assert_cmd 2.1.2 for testing
- https://github.com/crossterm-rs/crossterm — Verified crossterm 0.28 as alternative
- https://github.com/rust-cli/human-panic — Checked human-panic for error handling
- https://rust-cli.github.io/book/ — Rust CLI book for best practices

---
*Stack research for: CLI Development Tools in Rust*
*Researched: 2026-02-19*