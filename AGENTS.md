# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Project Purpose

Loa'a is a chore and rewards tracking system that helps kids stay motivated through a gamified task completion system with monetary rewards.

**Key Documentation:**
- `docs/vision.md` - Problem statement, goals, and philosophy
- `docs/roadmap.md` - Phased rollout plan and timeline
- `docs/architecture.md` - Technical decisions and structure

**Current Phase:** Phase 1 (MVP) - Single-user local application

## Working on This Project

### Before You Start

1. **Read the docs** - Start with `docs/vision.md` and `docs/roadmap.md` to understand context
2. **Check current phase** - We're building Phase 1 MVP features only
3. **Review Priority 1 issues** - These are foundational work
4. **Understand the scope** - Don't build Phase 2+ features yet

### Development Principles

**Start Simple:**
- Implement the minimal working version first
- Don't prematurely optimize
- Validate core mechanics before adding complexity

**Local-First:**
- Everything runs on the parent's laptop
- No deployment complexity for MVP
- Embedded database (no server process)

**Rust Best Practices:**
- Use `anyhow` for error handling with context
- Prefer explicit types over type inference where it aids clarity
- Write tests for business logic in `loaa-core`
- Keep web UI and core logic separate

**Tech Stack:**
- **Leptos** for web (SSR + hydration)
- **SurrealDB** for database (embedded mode)
- **Workspace structure** (core/web/mcp)

### Code Quality Standards

**Always Fix Compilation Errors and Warnings:**
- **Zero tolerance policy**: All code must compile without errors or warnings
- Never leave compilation errors "for later" - fix them immediately
- Treat warnings as errors - they often indicate real problems
- Use `cargo clippy` and `cargo fmt` to maintain code quality
- Run `cargo check` frequently during development

**Code Quality Philosophy:**
- **Do the right thing**: Prioritize clean code and optimal architecture over speed
- **No shortcuts**: Take whatever time is necessary to make correct decisions
- **Sustainable pace**: Better to do it right once than fix it multiple times

**When Working on Issues:**

**Discovering Improvements:**
- If you notice something that should be cleaned up or refactored while working on an issue:
  1. Create a new issue for the improvement
  2. Continue with your current work
  3. Address the improvement in a separate issue later

**Substantial Refactoring:**
- If you realize a substantial refactor would greatly simplify your current work:
  1. **Stash your current changes** (or commit to a WIP branch)
  2. **Create a new issue** for the refactoring work
  3. **Complete the refactoring** - implement, test, commit, and close the issue
  4. **Return to original work** - you may need to start over, but the codebase is now cleaner
  5. The refactoring makes the original work easier and more correct

**Why This Approach:**
- Keeps the codebase clean and maintainable
- Prevents accumulating technical debt
- Makes future work easier
- Ensures each commit is meaningful and correct
- Maintains architectural integrity

### What NOT to Build (Yet)

These are explicitly deferred to later phases:

**Phase 2+ Features:**
- Multi-user authentication
- Kids logging in themselves
- Approval workflows
- Task prerequisites
- Dynamic bounties
- Rewards redemption

If you find yourself implementing these, stop and check the roadmap.

### Money Handling

**Critical:** Use `rust_decimal::Decimal` for all monetary values, never floats.

```rust
use rust_decimal::Decimal;

// Good
let value = Decimal::from_str("1.50")?;

// Bad - DO NOT DO THIS
let value = 1.5_f64;  // Causes rounding errors!
```

### Data Model Guidelines

**Phase 1 Entities:**
- `Kid` - A child in the household
- `Task` - A chore with a value and cadence
- `LedgerEntry` - A transaction (earned/adjusted)

See `docs/architecture.md` for detailed data model documentation.

### Workspace Structure

```
loaa/
├── crates/
│   ├── core/       # Business logic, data models, database
│   ├── web/        # Leptos web application
│   └── mcp/        # MCP server for AI integration
```

**Import Rules:**
- `core` → depends on nothing (pure business logic)
- `web` → depends on `core`
- `mcp` → depends on `core`
- Never import web or mcp into core

### Commit Guidelines

Follow strict commit message format:

**Format:**
- **Title only** - No body, no additional content
- **Imperative mood** - "Add feature" not "Added feature" or "Adds feature"
- **Capitalize first letter** - "Fix bug" not "fix bug"
- **No period at end** - "Update docs" not "Update docs."
- **Under 50 characters** - Keep it concise and scannable
- **One logical change per commit** - Don't mix features

**Examples:**
- ✅ `Add user authentication`
- ✅ `Fix memory leak in worker pool`
- ✅ `Update dependencies to latest versions`
- ❌ `added user authentication` (not imperative, not capitalized)
- ❌ `Fixes the memory leak in the worker pool that was causing issues` (too long)
- ❌ `Update docs.` (has period)
- ❌ Any commit with body text or multiple lines

**Critical:** Use `git commit -m "Title"` NOT `git commit -m "$(cat <<EOF ...)"` with heredoc.
The title is the ONLY content. No attribution, no body, no Co-Authored-By, no emojis.

**Rationale:**
Short, imperative commits create a clean, scannable git history. Each commit should represent a single logical change that can be described in one concise line.

### Testing Strategy

**Unit Tests:**
- Test all business logic in `loaa-core`
- Test data model validation
- Test helper methods

**Integration Tests:**
- Test database operations
- Test full workflows (create task → complete → check balance)

**E2E Tests:**
- Not needed for MVP
- Add later when UI stabilizes

### When to Create Issues

Create new issues when you:
- Discover a bug while working
- Find missing functionality needed for current task
- Identify technical debt that should be addressed

### Pivot Points

Be aware of places where we might change direction:

**After Leptos setup:**
- If Leptos feels too complex → consider Axum + HTMX
- Document issues in the issue description

**After SurrealDB integration:**
- If SurrealDB causes problems → consider SQLite
- Keep database abstraction clean for easy swap

**During development:**
- If the concept doesn't resonate with the family → pivot
- Document learnings in `docs/`

### Getting Help

**Resources:**
- Check existing docs in `docs/`
- Consult architecture docs for technical decisions
- Look at Leptos book: https://book.leptos.dev/
- SurrealDB docs: https://surrealdb.com/docs

**Don't Assume:**
- When in doubt about scope, check roadmap
- When uncertain about design, check architecture docs
- When unclear about priorities, check issue priorities

### Managing AI-Generated Planning Documents

AI assistants often create planning and design documents during development:
- PLAN.md, IMPLEMENTATION.md, ARCHITECTURE.md
- DESIGN.md, CODEBASE_SUMMARY.md, INTEGRATION_PLAN.md
- TESTING_GUIDE.md, TECHNICAL_DESIGN.md, and similar files

**Best Practice: Use a dedicated directory for these ephemeral files**

**Recommended approach:**
- Create a `history/` directory in the project root
- Store ALL AI-generated planning/design docs in `history/`
- Keep the repository root clean and focused on permanent project files
- Only access `history/` when explicitly asked to review past planning

**Example .gitignore entry (optional):**
```
# AI planning documents (ephemeral)
history/
```

**Benefits:**
- ✅ Clean repository root
- ✅ Clear separation between ephemeral and permanent documentation
- ✅ Easy to exclude from version control if desired
- ✅ Preserves planning history for archeological research
- ✅ Reduces noise when browsing the project
