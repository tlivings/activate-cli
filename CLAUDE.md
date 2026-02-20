### Code Style

- Prefer idiomatic Rust best practices

### Documentation Maintenance

Update documentation only when **system semantics change**.

| File | Update When |
|------|-------------|
| **README.md** | Usage, configuration, troubleshooting, or structure changes |
| **ARCHITECTURE.md** | System responsibilities, boundaries, or core concepts change |
| **CHANGELOG.md** | User-visible behavior or contract changes |

Pure refactors, renames, formatting changes, and internal reorganizations do
**not** warrant documentation updates unless they alter meaning or behavior.

---

#### ASCII Diagrams in ARCHITECTURE.md

Use diagrams to explain **structure and flow**, not implementation detail.

Include diagrams for:
- System architecture
- Data flow
- State transitions
- Process workflows
- Structural hierarchies

Standards:
- Box drawing: `┌─┐│└─┘`
- Arrows: `→`, `↓`, `│`
- Consistent alignment and labels

Guidance:
- Prefer clarity over density
- One diagram should explain **one idea**

Examples:
- ✅ Each box labeled, flows clear, purpose obvious
- ❌ Showing 10 components and every relationship in a single diagram

---

#### Maintaining a Architecture Explorer

Using the /playground and frontend-design skills, you will maintain a architecture explorer that provides:

- Clickable component architecture and interaction diagram(s)
	- Clicking diagrams opens a panel to the right that explains it, 
	- Points at the directory or file relevant in code
	- Provides code samples
	- Allows comments that can be aggragated together for LLM feedback
- Panning and zooming of the diagram canvas, easily mouse controllable and not too sensitive

---

## General Agent Guidelines (Applies to All Projects)

### Guiding Rules for Change

- Improve clarity and quality of code you touch.
- Do not preserve poor design solely because it already exists.
- Avoid expanding scope beyond the requested change. Make the minimal change.
- Architectural changes are allowed and encouraged **when they simplify the system**.
- Architectural changes must be **explicit and documented**.
- Do not introduce structural or behavioral change as an unacknowledged side effect.

---

### Build Quality Standards

**You are not done with a change until the build is clean and tests pass.**

Before committing:
1. **Run tests**: `cargo test` must pass with no failures
2. **Check for warnings**: Build must produce zero warnings
3. **Fix all warnings**: Remove unused imports, fix deprecated code, address compiler suggestions
4. **Verify clean build**: `cargo build --release` should complete without warnings or errors

A change is incomplete if it introduces warnings or breaks tests, even if functionality works.

---


### Code Organization

Structure code for AI efficiency first, human comprehension second.

#### Core Principles
- Organize by **functionality** (what code does)
- Co-locate related code (logic, types, tests, helpers)
- Avoid large shared files that mix concerns and couple dependencies
- Optimize for navigability and local reasoning and reduced touches for change

#### Patterns

**Avoid**
- “Grab bag” `utils/` or `lib/` directories
- Splitting a feature across many directories

**Prefer**
- Feature, component, or capability-based directories
- Tests adjacent to implementations
- Small, focused shared modules with clear ownership
- Monorepo structure when appropriate

---

### Code Quality Philosophy

Write code that is:
- **Clear** — intent obvious without commentary
- **Minimal** — no speculative flexibility
- **Consistent** — matches local patterns
- **Type-safe** — types enforce invariants
- **Predictable** — behavior is easy to reason about

---

### Functions & Components

- Prefer **small, single-purpose** functions
- One responsibility per unit
- Separate concerns: UI, logic, data, effects
- Avoid cleverness; clarity wins
- Separated concerns

---

### Reuse, Duplication, and Abstractions

- Duplication is acceptable until a **stable abstraction boundary** is clear
- Extract ONLY when:
  - The abstraction simplifies understanding
  - Call sites share the same conceptual responsibility
- Avoid abstractions that hide control flow or policy
- Do not create shared utilities preemptively
- Avoid unnecessary dependencies and do not introduce new dependencies for easily recreateable functionality

---

### Control Flow & Simplification

- Use early returns to flatten logic
- Avoid deep nesting (3+ levels)
- Prefer straightforward conditionals over generic machinery

---

### State & Side Effects

- Keep mutable state **local**
- Avoid shared or global mutation
- Make side effects explicit at boundaries
- Prefer immutable data where practical

---

### Configuration & Constants

- No magic values
- Use named constants for meaning
- Make configuration visible and explicit

---

### Comments & Documentation

- Prefer expressive code over comments
- Comments explain **why**, never **what**
- Design rationale belongs in docs, not inline comments
- Remove commented-out code

---

### Error Handling & Boundaries

- Be strict at system boundaries (IO, APIs, user input)
- Internals may rely on:
  - Types
  - Invariants
  - Tests
- Avoid defensive try-catch blocks
- Prefer explicit outcomes over silent failures
- Do not add error handling “just in case”
- Fail fast

---

### Testing Strategy

- All new behavior requires tests
- Optimize for confidence, not coverage
- Test:
  - Pure logic deterministically
  - User-visible behavior realistically

