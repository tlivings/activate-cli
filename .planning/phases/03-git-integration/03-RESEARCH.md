# Phase 3: Git Integration - Research

**Researched:** 2026-02-20
**Domain:** Git repository cloning, status checking, TUI enhancements, documentation
**Confidence:** HIGH

## Summary

Phase 3 adds git integration for cloning repositories from URLs, detecting/storing git origins, and warning about uncommitted changes. The phase also includes TUI enhancements (deactivate/archive shortcuts, help panel) and README documentation.

The existing codebase already has git2 in its planned stack (STACK.md) and the database schema already includes a `git_origin` column. The TUI uses ratatui with crossterm and has a pattern for handling state changes via requests. The implementation should leverage git2 for all git operations rather than shelling out, and use ratatui's overlay pattern for the help panel.

**Primary recommendation:** Use git2 for cloning and status checks, parse URLs with string operations (SSH URLs aren't valid URLs), implement TUI help as a modal overlay with Clear widget, keep README simple and feature-focused.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Accept HTTPS and SSH URL formats (no shorthand like user/repo)
- Clone destination: always tracked_directory/<repo-name>
- Folder naming: repo name only, no user/ prefix nesting
- Name conflict: error and abort, suggest `activate <name>` if folder exists
- Warn on deactivate and archive only (not on activate)
- Non-blocking warning: show message but proceed anyway
- Scope: staged changes, unstaged changes, AND unpushed commits
- Warning format: summary counts only ("3 uncommitted changes, 2 unpushed commits")
- No "missing" state - if folder doesn't exist, auto-remove from database
- Detection happens during list, activate, and sync operations
- No reclone functionality - user deleted it, their problem
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

### Deferred Ideas (OUT OF SCOPE)
None - discussion stayed within phase scope

</user_constraints>

## Standard Stack

### Core (Already in Cargo.toml or planned)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.20.4 | Git operations | Comprehensive libgit2 bindings, already in planned stack, handles cloning/status/remotes |
| ratatui | 0.29 | TUI framework | Already in use, provides Clear widget for overlays |
| crossterm | 0.28 | Terminal handling | Already in use with ratatui |

### No New Dependencies Needed
The existing codebase has all required dependencies. Git URL parsing can be done with standard string operations since SSH URLs (git@github.com:user/repo) aren't valid URLs for the `url` crate anyway.

**Installation (add git2 to existing Cargo.toml):**
```bash
cargo add git2
```

## Architecture Patterns

### Recommended Module Structure
```
src/
├── git/                    # NEW: Git operations module
│   ├── mod.rs              # Module exports
│   ├── clone.rs            # URL parsing, cloning logic
│   ├── status.rs           # Staged/unstaged/unpushed checks
│   └── origin.rs           # Origin detection and storage
├── tui/
│   ├── app.rs              # ADD: deactivate/archive handlers, help toggle
│   ├── ui.rs               # ADD: help overlay rendering
│   └── help.rs             # NEW: Help content and keybinding definitions
├── commands/
│   ├── activate.rs         # MODIFY: handle URL input, trigger clone
│   ├── deactivate.rs       # MODIFY: add git warning check
│   ├── archive.rs          # MODIFY: add git warning check
│   ├── list.rs             # MODIFY: add --verbose flag, origin column
│   └── sync.rs             # MODIFY: detect origins, cleanup missing
└── README.md               # NEW: Project documentation
```

### Pattern 1: Git Module Encapsulation
**What:** Isolate all git2 operations in a dedicated module
**When to use:** All git operations throughout the codebase
**Example:**
```rust
// src/git/mod.rs
pub mod clone;
pub mod status;
pub mod origin;

pub use clone::clone_repository;
pub use status::GitStatus;
pub use origin::detect_origin;
```

### Pattern 2: URL Parsing for Git
**What:** Parse git URLs (both HTTPS and SSH) to extract repo name
**When to use:** When accepting URLs for clone operations
**Example:**
```rust
// Source: Standard string operations (SSH URLs aren't valid URLs)
pub fn extract_repo_name(url: &str) -> Option<String> {
    // Handle both formats:
    // https://github.com/user/repo.git
    // git@github.com:user/repo.git

    let name = url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit(|c| c == '/' || c == ':')
        .next()?;

    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
```

### Pattern 3: Git Status Summary
**What:** Collect git status into summary counts
**When to use:** Before deactivate/archive operations
**Example:**
```rust
// Source: git2 docs - Status struct
use git2::{Repository, Status};

pub struct GitStatus {
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub unpushed_count: usize,
}

impl GitStatus {
    pub fn check(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)?;
        let statuses = repo.statuses(None)?;

        let mut staged = 0;
        let mut unstaged = 0;

        for entry in statuses.iter() {
            let status = entry.status();
            // INDEX_* flags = staged changes
            if status.intersects(
                Status::INDEX_NEW | Status::INDEX_MODIFIED |
                Status::INDEX_DELETED | Status::INDEX_RENAMED
            ) {
                staged += 1;
            }
            // WT_* flags = working tree (unstaged) changes
            if status.intersects(
                Status::WT_MODIFIED | Status::WT_DELETED |
                Status::WT_RENAMED
            ) {
                unstaged += 1;
            }
        }

        let unpushed = count_unpushed_commits(&repo)?;

        Ok(GitStatus {
            staged_count: staged,
            unstaged_count: unstaged,
            unpushed_count: unpushed
        })
    }

    pub fn has_warnings(&self) -> bool {
        self.staged_count > 0 || self.unstaged_count > 0 || self.unpushed_count > 0
    }

    pub fn warning_message(&self) -> String {
        let changes = self.staged_count + self.unstaged_count;
        match (changes > 0, self.unpushed_count > 0) {
            (true, true) => format!(
                "{} uncommitted changes, {} unpushed commits",
                changes, self.unpushed_count
            ),
            (true, false) => format!("{} uncommitted changes", changes),
            (false, true) => format!("{} unpushed commits", self.unpushed_count),
            (false, false) => String::new(),
        }
    }
}
```

### Pattern 4: Unpushed Commits Detection
**What:** Count commits in HEAD not in upstream
**When to use:** As part of git status check
**Example:**
```rust
// Source: git2 Revwalk docs
fn count_unpushed_commits(repo: &Repository) -> Result<usize> {
    // Get HEAD commit
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(0), // No HEAD (empty repo)
    };

    let head_commit = head.peel_to_commit()?;

    // Find upstream tracking branch
    let branch = repo.find_branch(
        head.shorthand().unwrap_or("HEAD"),
        git2::BranchType::Local
    )?;

    let upstream = match branch.upstream() {
        Ok(u) => u,
        Err(_) => return Ok(0), // No upstream configured
    };

    let upstream_commit = upstream.get().peel_to_commit()?;

    // Count commits in HEAD but not in upstream
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_commit.id())?;
    revwalk.hide(upstream_commit.id())?;

    Ok(revwalk.count())
}
```

### Pattern 5: TUI Help Overlay
**What:** Modal help panel rendered on top of main content
**When to use:** When user presses '?' key
**Example:**
```rust
// Source: ratatui Clear widget docs
use ratatui::widgets::Clear;

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Calculate centered area (60% width, 70% height)
    let help_area = centered_rect(60, 70, area);

    // Clear the area first
    frame.render_widget(Clear, help_area);

    // Render help content
    let help_text = vec![
        Line::from("Keybindings").style(Style::default().bold()),
        Line::from(""),
        Line::from("  j/k or arrows  Navigate"),
        Line::from("  Enter          Activate project"),
        Line::from("  d              Deactivate project"),
        Line::from("  a              Archive project"),
        Line::from("  i              Toggle ignore"),
        Line::from("  I              Show/hide ignored"),
        Line::from("  ?              Toggle help"),
        Line::from("  Esc/Ctrl+C     Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::bordered().title("Help"));

    frame.render_widget(help, help_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width * percent_x / 100;
    let height = area.height * percent_y / 100;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    Rect::new(x, y, width, height)
}
```

### Pattern 6: TUI Action Requests
**What:** Queue actions from key handler, execute in main loop
**When to use:** For operations that need database access (already in use)
**Example:**
```rust
// Extend existing App struct
pub struct App {
    // ... existing fields ...
    pub show_help: bool,
    pub deactivate_request: Option<String>,
    pub archive_request: Option<String>,
}

// In handle_key
(KeyCode::Char('d'), KeyModifiers::NONE) if self.input.is_empty() => {
    if let Some(project) = self.selected_project() {
        self.deactivate_request = Some(project.name.clone());
    }
}
(KeyCode::Char('a'), KeyModifiers::NONE) if self.input.is_empty() => {
    if let Some(project) = self.selected_project() {
        self.archive_request = Some(project.name.clone());
    }
}
(KeyCode::Char('?'), _) => {
    self.show_help = !self.show_help;
}
```

### Anti-Patterns to Avoid
- **Shelling out to git:** Use git2 for all operations - it's faster and more reliable
- **Parsing SSH URLs with url crate:** SSH URLs (git@host:path) aren't valid URLs, use string ops
- **Blocking on clone:** Clone can be slow, show progress or at minimum explain what's happening
- **Hardcoding "origin":** Some repos use different remote names, check for tracking branch first

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Git cloning | Custom git command wrapper | git2::Repository::clone() | Handles auth, progress, errors properly |
| Status parsing | Shell git status --porcelain | git2::Repository::statuses() | Type-safe, no parsing needed |
| Upstream detection | Manual ref parsing | git2::Branch::upstream() | Handles all edge cases |
| SSH auth | Manual key file reading | git2::Cred::ssh_key_from_agent() | Uses system SSH agent |
| URL parsing | Full URL parser for git URLs | Simple string split | SSH URLs aren't valid URLs |

**Key insight:** git2 handles authentication callbacks automatically when using RepoBuilder with proper FetchOptions. For most users, ssh_key_from_agent() will work if they have ssh-agent running.

## Common Pitfalls

### Pitfall 1: SSH URL Format
**What goes wrong:** Treating SSH URLs as valid URLs for url crate parsing
**Why it happens:** git@github.com:user/repo looks URL-like but isn't
**How to avoid:** Use string splitting on '/' and ':' for repo name extraction
**Warning signs:** url::Url::parse() returning errors for SSH URLs

### Pitfall 2: Missing Upstream Branch
**What goes wrong:** Unpushed commit check fails when no upstream is configured
**Why it happens:** New branches, repos cloned without --set-upstream
**How to avoid:** Return 0 unpushed when no upstream exists (not an error)
**Warning signs:** Errors on freshly created branches

### Pitfall 3: Empty Repository
**What goes wrong:** Status checks fail on repos with no commits
**Why it happens:** HEAD doesn't exist in empty repos
**How to avoid:** Check for HEAD existence before status operations
**Warning signs:** Errors on `git init` without commits

### Pitfall 4: TUI State After Action
**What goes wrong:** Project list doesn't update after deactivate/archive
**Why it happens:** App state holds stale copy of projects
**How to avoid:** Update project state in App after successful DB operation
**Warning signs:** TUI shows old state until restart

### Pitfall 5: Clone Auth Failures
**What goes wrong:** Clone fails silently or with cryptic errors
**Why it happens:** SSH agent not running, wrong credentials
**How to avoid:** Provide clear error message with troubleshooting hints
**Warning signs:** "authentication failed" without guidance

## Code Examples

### Git Clone with Credential Handling
```rust
// Source: git2 RepoBuilder and Cred docs
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};

pub fn clone_repository(url: &str, dest: &Path) -> Result<Repository> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        // Try SSH agent first
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            let username = username_from_url.unwrap_or("git");
            return Cred::ssh_key_from_agent(username);
        }

        // Fall back to default credentials (for HTTPS with credential helper)
        if allowed_types.contains(git2::CredentialType::DEFAULT) {
            return Cred::default();
        }

        Err(git2::Error::from_str("no valid credentials found"))
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    builder.clone(url, dest)
        .context("Failed to clone repository")
}
```

### Origin Detection
```rust
// Source: git2 Remote docs
pub fn detect_origin(path: &Path) -> Option<String> {
    let repo = Repository::open(path).ok()?;

    // Try "origin" first (most common)
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            return Some(url.to_string());
        }
    }

    // Fall back to first remote
    let remotes = repo.remotes().ok()?;
    for name in remotes.iter().flatten() {
        if let Ok(remote) = repo.find_remote(name) {
            if let Some(url) = remote.url() {
                return Some(url.to_string());
            }
        }
    }

    None
}
```

### Activate with URL Detection
```rust
pub fn execute_activate(db: &Database, input: &str) -> Result<()> {
    // Check if input looks like a URL
    if input.starts_with("https://") ||
       input.starts_with("git@") ||
       input.starts_with("ssh://") {
        return clone_and_activate(db, input);
    }

    // Existing project activation logic...
}

fn clone_and_activate(db: &Database, url: &str) -> Result<()> {
    let config = Config::load()?;

    let repo_name = extract_repo_name(url)
        .ok_or_else(|| anyhow!("Could not extract repository name from URL"))?;

    let dest = config.tracked_directory.join(&repo_name);

    // Check for conflicts
    if dest.exists() {
        return Err(anyhow!(
            "Folder '{}' already exists. Use `activate {}` to activate existing project.",
            dest.display(), repo_name
        ));
    }

    // Check if name already tracked
    if get_project_by_name(&db.conn, &repo_name)?.is_some() {
        return Err(anyhow!(
            "Project '{}' already exists in database.",
            repo_name
        ));
    }

    eprintln!("Cloning {} into {}...", url, dest.display());
    clone_repository(url, &dest)?;

    // Add to database
    add_project_with_state(&db.conn, &repo_name, &dest, ProjectState::Active)?;

    // Store origin
    update_git_origin(&db.conn, &repo_name, Some(url))?;

    eprintln!("Cloned and activated '{}'", repo_name);
    println!("{}", dest.display());

    Ok(())
}
```

### Warning Check for Deactivate/Archive
```rust
pub fn execute_deactivate(db: &Database, name: &str) -> Result<()> {
    let project = find_project_or_error(db, name)?;

    // Check for uncommitted changes (non-blocking warning)
    if let Ok(status) = GitStatus::check(&project.path) {
        if status.has_warnings() {
            eprintln!("Warning: {}", status.warning_message());
        }
    }

    update_project_state(&db.conn, &project.name, ProjectState::Inactive)?;
    println!("Deactivated '{}'", project.name);
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Shell out to git | Use git2 crate | 2020+ | Faster, more reliable, no git binary needed |
| tui-rs | ratatui | 2023 | ratatui is actively maintained fork |
| Manual credential handling | git2 Cred::ssh_key_from_agent | Always | Uses system SSH agent automatically |

**Deprecated/outdated:**
- tui-rs: Use ratatui instead (actively maintained)
- Parsing git status --porcelain: Use git2::statuses() instead

## Open Questions

1. **Clone progress indication**
   - What we know: git2 supports progress callbacks
   - What's unclear: Best UX for terminal - spinner vs percentage
   - Recommendation: Use simple "Cloning..." message for now, add progress later if needed

2. **Auth failure handling**
   - What we know: SSH agent auth can fail silently
   - What's unclear: How to guide users to fix SSH issues
   - Recommendation: Catch auth errors and suggest "Check SSH agent with ssh-add -l"

3. **Large repo performance**
   - What we know: Status checks can be slow on large repos
   - What's unclear: Acceptable threshold before showing warning
   - Recommendation: Cache status for --verbose list, check live only for deactivate/archive

## Sources

### Primary (HIGH confidence)
- git2 0.20.4 docs - Clone, Status, Cred, Repository structs
- ratatui 0.29 docs - Clear widget, Layout system
- Existing codebase - TUI patterns, database schema, command structure

### Secondary (MEDIUM confidence)
- .planning/research/STACK.md - Verified git2 as planned dependency
- .planning/research/PITFALLS.md - Git integration assumptions pitfall
- .planning/research/ARCHITECTURE.md - Module organization patterns

### Tertiary (LOW confidence)
- bat README structure - For README organization reference

## Metadata

**Confidence breakdown:**
- Git operations (clone, status, origin): HIGH - git2 docs are comprehensive
- TUI overlay pattern: HIGH - ratatui Clear widget documented
- URL parsing: HIGH - Simple string operations, tested patterns
- README structure: MEDIUM - Based on popular projects, adaptable

**Research date:** 2026-02-20
**Valid until:** 2026-03-20 (30 days - stable libraries)
