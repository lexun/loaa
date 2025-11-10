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
  1. Create a new beads issue for the improvement
  2. Link it with `discovered-from` dependency to the current issue
  3. Continue with your current work
  4. Address the improvement in a separate issue later

**Substantial Refactoring:**
- If you realize a substantial refactor would greatly simplify your current work:
  1. **Stash your current changes** (or commit to a WIP branch)
  2. **Create a new beads issue** for the refactoring work
  3. **Complete the refactoring** - implement, test, commit, and close the issue
  4. **Return to original work** - you may need to start over, but the codebase is now cleaner
  5. The refactoring makes the original work easier and more correct

**Why This Approach:**
- Keeps the codebase clean and maintainable
- Prevents accumulating technical debt
- Makes future work easier
- Ensures each commit is meaningful and correct
- Maintains architectural integrity

**Example Workflow:**
```bash
# Working on loaa-123, discover need for refactor
git stash
bd create "Refactor database connection handling" -p 1 --deps discovered-from:loaa-123
bd update loaa-456 --status in_progress
# ... do refactoring ...
bd close loaa-456 --reason "Completed refactoring"
git stash pop  # or start fresh if needed
# Continue with loaa-123
```

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
- **Imperative mood**: "Add feature" not "Added feature"
- **Capitalize first letter**: "Fix bug" not "fix bug"
- **No period at end**: "Update docs" not "Update docs."
- **Under 50 characters**: Keep it concise
- **One logical change per commit**: Don't mix features

See CLAUDE.md for full commit guidelines.

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

Link new issues to their parent:
```bash
bd create "Fix validation bug" -p 1 --deps discovered-from:loaa-ycs
```

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
- Review related beads issues
- Consult architecture docs for technical decisions
- Look at Leptos book: https://book.leptos.dev/
- SurrealDB docs: https://surrealdb.com/docs

**Don't Assume:**
- When in doubt about scope, check roadmap
- When uncertain about design, check architecture docs
- When unclear about priorities, check issue priorities

## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Auto-syncs to JSONL for version control
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**
```bash
bd ready --json
```

**Create new issues:**
```bash
bd create "Issue title" -t bug|feature|task -p 0-4 --json
bd create "Issue title" -p 1 --deps discovered-from:loaa-123 --json
```

**Claim and update:**
```bash
bd update loaa-42 --status in_progress --json
bd update loaa-42 --priority 1 --json
```

**Complete work:**
```bash
bd close loaa-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task**: `bd update <id> --status in_progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`
6. **Commit together**: Always commit the `.beads/issues.jsonl` file together with the code changes so issue state stays in sync with code state

### Auto-Sync

bd automatically syncs with git:
- Exports to `.beads/beads.jsonl` after changes (5s debounce)
- Imports from JSONL when newer (e.g., after `git pull`)
- No manual export/import needed!

### MCP Server (Recommended)

If using Claude or MCP-compatible clients, install the beads MCP server:

```bash
pip install beads-mcp
```

Add to MCP config (e.g., `~/.config/claude/config.json`):
```json
{
  "beads": {
    "command": "beads-mcp",
    "args": []
  }
}
```

Then use `mcp__beads__*` functions instead of CLI commands.

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

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ✅ Store AI planning docs in `history/` directory
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems
- ❌ Do NOT clutter repo root with planning documents

For more details, see README.md and QUICKSTART.md.
