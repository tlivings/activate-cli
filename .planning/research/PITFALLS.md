# Pitfalls Research

**Domain:** Rust CLI Tool Development (Project State Management)
**Researched:** 2026-02-19
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Shell Integration String Escaping Failures

**What goes wrong:**
Shell integration code (for directory changes, environment variables) breaks with special characters in paths, project names, or user input. Spaces, quotes, and shell metacharacters cause command injection vulnerabilities or silent failures.

**Why it happens:**
Developers assume simple string concatenation is sufficient for shell commands. Rust's strong typing doesn't protect you once you're generating shell code strings. Each shell (bash, zsh, fish, PowerShell) has different escaping rules.

**How to avoid:**
- Use shell-specific escaping libraries (shell-escape crate for Unix, dunce for Windows paths)
- Never use format!() or string concatenation for shell commands
- Generate shell functions/aliases using proper quoting mechanisms
- Test with paths containing spaces, quotes, $, `, \, and Unicode

**Warning signs:**
- Using format!("cd {}", path) anywhere in the code
- Shell integration works in testing but fails for some users
- Bug reports mentioning "weird characters" in project names

**Phase to address:**
Shell Integration Phase - must be designed correctly from the start

---

### Pitfall 2: Database File Locking on Windows

**What goes wrong:**
SQLite/embedded databases fail with "database is locked" errors on Windows due to different file locking semantics. Virus scanners and backup software can hold locks. Multiple CLI invocations can't coordinate.

**Why it happens:**
Unix allows multiple readers with advisory locking. Windows enforces mandatory locks. Developers test on Mac/Linux and ship broken Windows support.

**How to avoid:**
- Use WAL mode for SQLite with proper busy timeout (at least 5 seconds)
- Implement exponential backoff for database operations
- Consider using a lock file separate from the database
- Use rusqlite with bundled SQLite for consistent behavior

**Warning signs:**
- Works on Mac/Linux, intermittent failures on Windows
- "Database is locked" errors in user reports
- CI passes but users report data corruption

**Phase to address:**
Database Design Phase - architecture decision

---

### Pitfall 3: Inconsistent Path Normalization

**What goes wrong:**
The same project gets multiple database entries due to path inconsistencies: /Users/bob/project vs ~/project vs /Users/bob/./project vs /Users/bob/project/ (trailing slash). Symlinks create duplicate entries.

**Why it happens:**
Paths come from different sources (user input, pwd, git remote URLs) in different formats. Rust's PathBuf doesn't automatically canonicalize.

**How to avoid:**
- Always canonicalize paths before database storage using std::fs::canonicalize
- Handle canonicalization failures gracefully (deleted directories, permission issues)
- Store both canonical and display paths
- Use path-clean crate for cross-platform normalization

**Warning signs:**
- Duplicate projects in list output
- "Project not found" despite being visible in list
- Symlink-related bug reports

**Phase to address:**
Core Data Model Phase - fundamental to all operations

---

### Pitfall 4: Blocking Filesystem Operations Kill Performance

**What goes wrong:**
Scanning large directories or network mounts blocks the CLI for minutes. Users Ctrl+C out of frustration. Git operations on large repos timeout. The tool feels "slow" even for simple operations.

**Why it happens:**
Synchronous filesystem walks, especially with metadata collection. Network filesystems multiply latency. Git operations are inherently slow on large repos.

**How to avoid:**
- Use async filesystem operations with tokio
- Implement progress indicators for operations > 100ms
- Add --depth limits for filesystem scanning
- Cache filesystem state with inotify/FSEvents/ReadDirectoryChangesW
- Use gitoxide instead of calling git binary for better performance

**Warning signs:**
- "Hangs" when scanning home directory
- No feedback during long operations
- Users complain about responsiveness

**Phase to address:**
Performance Optimization Phase - but design for async from start

---

### Pitfall 5: Git Integration Assumes Too Much

**What goes wrong:**
Tool fails for users with: non-standard git configs, worktrees, submodules, bare repos, or different git versions. Assumes 'origin' remote exists and is GitHub.

**Why it happens:**
Developers test with simple git setups. Git has hundreds of configuration options and edge cases. GitHub-centric assumptions exclude GitLab/Bitbucket users.

**How to avoid:**
- Use git2-rs library instead of shelling out to git
- Handle missing remotes, multiple remotes, non-origin defaults
- Support worktrees and submodules explicitly
- Test with git versions back to 2.25 (Ubuntu 20.04 LTS)

**Warning signs:**
- "Works on my machine" for git features
- Breaks for users with custom git workflows
- Hardcoded "origin" or "github.com" strings

**Phase to address:**
Git Integration Phase - needs comprehensive git scenario testing

---

### Pitfall 6: Panic in Production from Unwraps

**What goes wrong:**
CLI crashes with unhelpful "thread 'main' panicked" messages. Users see stack traces instead of error messages. One bad project in database crashes entire tool.

**Why it happens:**
Liberal use of unwrap()/expect() during development. Error handling added as afterthought. Rust makes panicking easy with ? operator misuse.

**How to avoid:**
- Use anyhow/thiserror for error handling from day one
- Set panic = "abort" in release profile
- Add #![warn(clippy::unwrap_used)] to main.rs
- Return Result<T> from main with proper error formatting
- Use expect() only with clear messages for true invariants

**Warning signs:**
- unwrap() calls in code review
- Bug reports with stack traces
- "It just crashed" without error message

**Phase to address:**
Initial Setup Phase - establish error handling patterns early

---

### Pitfall 7: Config File Format Lock-in

**What goes wrong:**
Choosing wrong config format (TOML/JSON/YAML) causes migration pain. Users can't hand-edit configs. No backwards compatibility. Config parsing adds huge dependencies.

**Why it happens:**
Picking format based on personal preference rather than use case. Not considering human editing, schema evolution, or dependency weight.

**How to avoid:**
- Use TOML for human-editable configs (not JSON)
- Design versioned config schema from start
- Implement migration logic for schema changes
- Keep config minimal - database for complex state
- Use serde with deny_unknown_fields for strict parsing

**Warning signs:**
- Users editing JSON by hand and breaking it
- Can't add new config fields without breaking old versions
- Config parser is 50% of binary size

**Phase to address:**
Configuration Design Phase - hard to change later

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| String-based shell generation | Quick implementation | Security vulnerabilities, platform bugs | Never - use proper escaping |
| Synchronous filesystem ops | Simpler code | Unresponsive CLI, can't cancel operations | Only for < 10ms operations |
| Hardcoded paths | Works locally | Breaks on different systems | Never in production |
| println! debugging | Fast debugging | Pollutes output, breaks scripting | Only in debug builds |
| Skipping Windows CI | Faster CI | Windows users get broken builds | Never - Windows is 70% of users |
| Git command shelling | Avoid git2-rs complexity | Slow, version-dependent, parsing errors | Only for MVP prototype |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Shell | Assuming bash is available | Detect shell, provide shell-specific integration |
| Git | Assuming git is in PATH | Bundle git or use git2-rs library |
| GitHub API | No rate limiting | Implement backoff, cache responses, use GraphQL |
| SQLite | Default settings | Enable WAL, foreign keys, busy timeout |
| File watching | Using polling | Use platform-specific APIs (inotify/FSEvents/etc) |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Loading entire database into memory | Fast for small datasets | Use iterators, pagination | > 1000 projects |
| Scanning filesystem recursively | Works for single project | Add depth limits, exclusions | Large monorepos |
| Synchronous git operations | Fine for small repos | Use async, add progress bars | > 1GB repos |
| No caching of expensive operations | Repeated computation | Cache git status, file metadata | > 100 projects |
| Regex compilation in hot loops | Slow pattern matching | Compile once with lazy_static | Any scale |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Unescaped shell commands | Command injection | Use shell-escape crate |
| Following symlinks blindly | Directory traversal | Use canonicalize, check boundaries |
| Git clone without validation | Malicious repos | Validate URLs, use --depth=1 |
| World-readable database | Credential exposure | Set 0600 permissions |
| Trusting project names | Path injection | Sanitize, use IDs internally |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No progress indication | Users think it's frozen | Show spinners for > 100ms operations |
| Unclear error messages | Users can't fix issues | Provide actionable error messages |
| No --dry-run option | Users afraid to run commands | Add dry-run for destructive operations |
| Breaking changes without warning | Workflows break on update | Semantic versioning, deprecation warnings |
| No shell completion | Poor discoverability | Generate completions for all shells |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Shell integration:** Often missing fish/PowerShell — verify all shells work
- [ ] **Project discovery:** Often missing exclusions — verify .git, node_modules, target ignored
- [ ] **Database migrations:** Often missing rollback — verify schema upgrades work
- [ ] **Error messages:** Often missing context — verify errors guide users to solutions
- [ ] **Windows support:** Often missing path handling — verify UNC paths, long paths work
- [ ] **Git integration:** Often missing worktree support — verify non-standard setups work
- [ ] **Async operations:** Often missing cancellation — verify Ctrl+C works cleanly

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Database corruption | MEDIUM | Add repair command, backup before migrations |
| Shell integration breaks | LOW | Provide uninstall command, document manual cleanup |
| Path normalization bugs | HIGH | Migration script to fix duplicates, add uniqueness constraint |
| Performance regression | MEDIUM | Add profiling, provide --fast mode with reduced features |
| Git state confusion | LOW | Add refresh command, document manual fix |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Shell escaping | Shell Integration | Test with special characters in CI |
| Database locking | Database Design | Windows CI with concurrent access |
| Path normalization | Core Data Model | Unit tests with edge cases |
| Blocking operations | Architecture Design | Benchmark tests in CI |
| Git assumptions | Git Integration | Test matrix of git setups |
| Production panics | Initial Setup | Clippy deny unwrap in CI |
| Config format | Configuration Design | Schema migration tests |

## Sources

- Rust CLI WG recommendations and anti-patterns
- ripgrep, bat, exa codebases for best practices
- Common issues in cargo, rustup issue trackers
- SQLite documentation on locking and WAL mode
- Shell escaping CVEs in various CLI tools
- Personal experience with cross-platform Rust CLI tools

---
*Pitfalls research for: Rust CLI Tool Development*
*Researched: 2026-02-19*