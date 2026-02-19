# Feature Research

**Domain:** CLI Project Management/Workspace Tools
**Researched:** 2026-02-19
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Quick navigation to project directories | Core value prop - users install these tools to save time | LOW | Must be faster than cd/ls navigation |
| Project listing/discovery | Need to see what projects exist | LOW | Simple list command with basic filtering |
| Add/remove projects | Basic CRUD operations | LOW | Manual registration or auto-discovery |
| Shell integration (cd on activate) | Users expect seamless shell experience | MEDIUM | Requires shell hooks/functions |
| Fuzzy/partial matching | Modern CLI UX standard (fzf, zoxide) | MEDIUM | Users expect "proj" to match "project-name" |
| Recent/frecency sorting | Smart defaults save keystrokes | MEDIUM | Track usage patterns, not just alphabetical |
| Config file support | Standard for CLI tools | LOW | YAML/TOML/JSON for settings |
| Multiple project paths | Developers work across ~/work, ~/personal, etc. | LOW | Support multiple root directories |
| Basic status display | Show if project exists, path is valid | LOW | Prevent navigation to deleted directories |
| Tab completion | Expected CLI ergonomics | MEDIUM | Shell-specific completion scripts |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Project state management (active/inactive/archived) | Unique mental model for project lifecycle | MEDIUM | Core differentiator for "activate" |
| GitHub integration (clone, status, PR info) | Reduces context switching | HIGH | API integration, auth handling |
| Tmux session auto-creation/attachment | Deep workflow integration | HIGH | Manages terminal multiplexer complexity |
| Project templates/scaffolding | Quick project initialization | MEDIUM | "activate new --template=node" |
| Environment auto-loading (direnv-style) | Project-specific configs without direnv | MEDIUM | Load .env, set PATH, etc. |
| Project metadata/tags | Organization beyond directories | MEDIUM | Tag projects by client, language, status |
| Batch operations | Manage multiple projects at once | MEDIUM | "activate archive --tag=client-x" |
| Activity tracking/analytics | "Which projects am I neglecting?" | MEDIUM | Last accessed, time spent |
| Project dependencies | "This project needs services X, Y" | HIGH | Docker compose, service orchestration |
| Smart suggestions | "You haven't touched X in 30 days" | MEDIUM | Proactive project management |
| Cross-machine sync | Same project list everywhere | HIGH | Cloud storage or git-based sync |
| IDE/editor integration | Open project in preferred editor | LOW | "activate open [project] --editor=vscode" |
| Task integration | Built-in todo tracking per project | HIGH | Could integrate with GitHub Issues |
| Project health checks | "Is this project's deps up to date?" | MEDIUM | Run linters, security checks |
| Quick project switching hotkeys | Global hotkeys for instant switching | HIGH | OS-specific complexity |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Automatic project discovery (scan filesystem) | "Find all my projects automatically" | Slow, creates noise, git repos everywhere | Explicit add with optional scan command |
| Complex query DSL | "I want SQL-like project queries" | Overkill for typical use, steep learning curve | Simple fuzzy search with basic filters |
| GUI/TUI as primary interface | "Modern apps need visual interfaces" | Breaks CLI workflow, slower than commands | Optional TUI for discovery, not required |
| Built-in VCS operations | "Do git operations through activate" | Reinventing git poorly, scope creep | Integrate with git, don't replace it |
| Project file watching | "Auto-update when files change" | Performance overhead, battery drain | On-demand status checks |
| Social features | "Share projects with team" | Privacy concerns, enterprise firewall issues | Focus on local-first, optional sharing |
| AI-powered suggestions | "AI should organize my projects" | Overcomplicated, privacy concerns | Simple heuristics for suggestions |
| Embedded scripting language | "Custom project logic in Lua/JS" | Complexity explosion, security risks | Shell scripts or simple hooks |
| Database-backed storage | "SQLite for project metadata" | Overkill, harder backup/sync | Plain text files (YAML/TOML) |
| Plugin system | "Extensible architecture" | Maintenance burden, compatibility issues | Focus on core features, shell integration |

## Feature Dependencies

```
[Project state management]
    └──requires──> [Project listing]
                       └──requires──> [Add/remove projects]

[GitHub integration] ──requires──> [Project metadata]
                                       └──requires──> [Config file support]

[Tmux integration] ──enhances──> [Project state management]

[Environment auto-loading] ──enhances──> [Shell integration]

[Activity tracking] ──requires──> [Project state management]

[Smart suggestions] ──requires──> [Activity tracking]

[Project templates] ──conflicts──> [Automatic discovery]
```

### Dependency Notes

- **State management requires listing:** Can't manage state without knowing what projects exist
- **GitHub integration requires metadata:** Need to store repo URLs, tokens per project
- **Tmux enhances state management:** Active projects get tmux sessions
- **Environment loading enhances shell integration:** Deeper integration beyond just cd
- **Activity tracking requires state:** Need to track when projects become active/inactive
- **Smart suggestions need activity data:** Recommendations based on usage patterns
- **Templates conflict with auto-discovery:** Explicit creation vs. automatic finding

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] Add/remove/list projects — Core CRUD operations
- [ ] Quick navigation with fuzzy matching — Primary value proposition
- [ ] Project state (active/inactive/archived) — Core differentiator
- [ ] Shell integration (cd on activate) — Expected UX
- [ ] Config file support — Standard for CLI tools
- [ ] Recent/frecency sorting — Smart defaults
- [ ] Basic GitHub integration (clone, open) — Key differentiator

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] Tmux session management — When users request deeper integration
- [ ] Project metadata/tags — When users need organization
- [ ] Environment auto-loading — When direnv integration requested
- [ ] Activity tracking — When users want insights
- [ ] Project templates — When initialization friction identified
- [ ] Batch operations — When managing many projects

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Cross-machine sync — Complex, wait for user demand
- [ ] Task integration — Scope expansion, validate need first
- [ ] Project health checks — Nice to have, not core
- [ ] Smart suggestions — Requires usage data to be valuable
- [ ] IDE deep integration — Per-editor complexity

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Quick navigation | HIGH | LOW | P1 |
| Project state management | HIGH | MEDIUM | P1 |
| Shell integration | HIGH | MEDIUM | P1 |
| Fuzzy matching | HIGH | MEDIUM | P1 |
| GitHub clone/open | HIGH | MEDIUM | P1 |
| Project listing | HIGH | LOW | P1 |
| Config file | MEDIUM | LOW | P1 |
| Frecency sorting | HIGH | MEDIUM | P1 |
| Tmux integration | MEDIUM | HIGH | P2 |
| Project metadata | MEDIUM | MEDIUM | P2 |
| Environment loading | MEDIUM | MEDIUM | P2 |
| Activity tracking | MEDIUM | MEDIUM | P2 |
| Project templates | MEDIUM | MEDIUM | P2 |
| Batch operations | LOW | MEDIUM | P3 |
| Cross-machine sync | LOW | HIGH | P3 |
| Task integration | LOW | HIGH | P3 |
| Smart suggestions | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | zoxide | tmux/tmuxinator | todo.txt/taskwarrior | Our Approach |
|---------|--------|-----------------|---------------------|--------------|
| Quick navigation | Frecency algorithm | N/A | N/A | Frecency + state awareness |
| Project organization | Just directories | Sessions/windows | Tasks/projects | State-based lifecycle |
| Persistence | Usage database | Running sessions | Task database | State + metadata files |
| GitHub integration | No | No | No | Native integration |
| Environment management | No | No | No | Optional .env loading |
| Workflow automation | No | YAML configs | Task dependencies | Simple hooks |
| Multi-project ops | No | No | Bulk task ops | Batch state changes |
| Analytics | Basic frecency | No | Task reports | Activity insights |

## Sources

- GitHub: tmux, tmuxinator, zoxide, z, fzf (navigation and workspace tools)
- GitHub: taskwarrior, todo.txt-cli (task management patterns)
- GitHub: direnv, starship (environment and context awareness)
- GitHub: just (project command organization)
- Analysis based on common patterns across 10+ CLI project tools

---
*Feature research for: CLI Project Management/Workspace Tools*
*Researched: 2026-02-19*