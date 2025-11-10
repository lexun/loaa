# Loa'a Architecture

## Overview

Loa'a is a full-stack Rust application using modern web technologies. The architecture prioritizes:
- **Type safety** - Rust everywhere, from database to UI
- **Developer experience** - Tools that are enjoyable to work with
- **Local-first** - No external dependencies, embedded database
- **Simplicity** - Start simple, add complexity only when needed

## Tech Stack

### Web Framework: Leptos

**What**: Modern full-stack Rust web framework with fine-grained reactivity

**Why Leptos:**
- Full-stack Rust (server + client in one codebase)
- Server-side rendering (SSR) with hydration
- Reactive signals (like SolidJS) for efficient UI updates
- Compile-time optimizations via macros
- WASM for client-side code
- Growing ecosystem

**Alternatives Considered:**
- **Axum + HTMX**: Simpler, more traditional, but less interactive
- **Dioxus**: Similar to Leptos but different philosophy
- **Yew**: Pure frontend, would need separate backend

**Decision**: Start with Leptos to learn modern Rust web development. If it becomes too complex, we can pivot to Axum + HTMX in later phases.

### Database: SurrealDB

**What**: Modern multi-model database with embedded mode

**Why SurrealDB:**
- **Embedded mode** - Single binary, no server process
- **Graph relations** - Good for task prerequisites (Phase 3)
- **Native Rust** - Written in Rust, async-first
- **Schema flexibility** - Easy to iterate on data models
- **LiveQuery** - Real-time updates (useful later)

**Alternatives Considered:**
- **SQLite**: More mature, battle-tested, better tooling
- **PostgreSQL**: Most features, but requires server
- **Redb**: Pure Rust embedded, but minimal features

**Decision**: Experiment with SurrealDB for learning. SQLite is a solid fallback if we hit issues.

**Migration Plan**: Keep data access layer abstracted in `loaa-core` so we can swap databases if needed.

### MCP Server

**What**: Model Context Protocol server allowing AI assistants to interact with Loa'a

**Why MCP:**
- Parent can manage tasks via natural language
- Reduces friction ("Claude, mark homework complete for Alice")
- Query system state without opening browser
- Future: kids could use AI to understand their tasks

**Implementation**: Separate binary (`loaa-mcp`) that uses `loaa-core` for business logic.

## Project Structure

```
loaa/
├── crates/
│   ├── core/           # Domain models, business logic
│   ├── web/            # Leptos web application
│   └── mcp/            # MCP server
├── docs/               # Design documentation
└── .beads/             # Issue tracking
```

### Workspace Organization

**Benefits:**
- Shared domain types across all binaries
- Clear separation of concerns
- Easy to test business logic independent of UI
- Can add more binaries later (CLI tools, etc.)

**Inspired by**: The `nova` project's workspace structure

## Crate Responsibilities

### `loaa-core`

**Purpose**: Pure business logic and domain models

**Contains:**
- Data models (Task, Kid, LedgerEntry, etc.)
- Database abstraction layer
- Business rules (ledger calculations, task validation)
- No HTTP, no UI code

**Dependencies:**
- SurrealDB client
- serde for serialization
- anyhow for error handling

**Why separate core:**
- Testable without spinning up web server
- Reusable across web and MCP binaries
- Forces clean architecture (no UI concerns in business logic)

### `loaa-web`

**Purpose**: Leptos web application (SSR + client)

**Contains:**
- Leptos components
- Server functions (API endpoints)
- Routing
- UI state management
- CSS/styling

**Dependencies:**
- Leptos
- Axum (for SSR server)
- loaa-core

**Architecture:**
- Server-side rendering for fast initial load
- Hydrates to interactive SPA
- Server functions handle API calls
- Shared types between server and client

### `loaa-mcp`

**Purpose**: MCP protocol server for AI assistant integration

**Contains:**
- MCP protocol implementation
- Tool definitions (complete_task, list_pending, etc.)
- JSON-RPC server

**Dependencies:**
- MCP SDK (if available, or custom implementation)
- loaa-core

**Tools to expose:**
- `list_tasks` - Get all tasks
- `complete_task` - Mark task complete
- `get_balance` - Get kid's balance
- `create_task` - Add new task
- `list_pending` - Show pending approvals (Phase 2)

## Data Models

### Core Entities (Phase 1)

```rust
struct Kid {
    id: Uuid,
    name: String,
    // Later: username, password_hash
}

struct Task {
    id: Uuid,
    name: String,
    description: String,
    value: Decimal,        // In dollars
    cadence: Cadence,      // Daily, Weekly, OneTime
    last_reset: DateTime,
}

enum Cadence {
    Daily,
    Weekly,
    OneTime,
}

struct LedgerEntry {
    id: Uuid,
    kid_id: Uuid,
    amount: Decimal,
    entry_type: EntryType,
    description: String,
    created_at: DateTime,
}

enum EntryType {
    Earned,    // From task completion
    Adjusted,  // Manual correction
    // Later: Redeemed (for rewards)
}
```

### Future Entities

```rust
// Phase 2: Authentication
struct User {
    id: Uuid,
    username: String,
    password_hash: String,
    role: UserRole,
    kid_id: Option<Uuid>,  // Links to Kid if role is Child
}

enum UserRole {
    Parent,
    Child,
}

// Phase 2: Approval Workflow
struct TaskCompletion {
    id: Uuid,
    task_id: Uuid,
    kid_id: Uuid,
    completed_at: DateTime,
    status: CompletionStatus,
    approved_at: Option<DateTime>,
    value_earned: Decimal,
}

enum CompletionStatus {
    Pending,
    Approved,
    Rejected,
}

// Phase 3: Prerequisites
struct TaskPrerequisite {
    task_id: Uuid,
    requires_task_id: Uuid,
}

// Phase 4: Bounties
struct BountyRule {
    task_id: Uuid,
    base_value: Decimal,
    increment: Decimal,
    interval: Duration,  // How often to increase
    max_value: Decimal,
}

// Phase 5: Rewards
struct Reward {
    id: Uuid,
    name: String,
    description: String,
    cost: Decimal,
    reward_type: RewardType,
}

enum RewardType {
    ScreenTime { minutes: u32 },
    Cash,
    Privilege { description: String },
}
```

## Database Schema Strategy

**Approach**: Start schemaless, add validation progressively

SurrealDB supports:
- Schemaless mode (flexible, fast iteration)
- Schemafull mode (strict validation)
- Hybrid (some tables strict, others flexible)

**Phase 1**: Schemaless for rapid iteration
**Later**: Add schema validation as models stabilize

**Migration strategy**: SurrealDB doesn't have traditional migrations. We'll use:
- Version field in each record
- Application-level migrations on startup
- Database dumps for backups

## Deployment Model

### Phase 1: Local Development

**How to run:**
```bash
cd loaa
cargo run -p loaa-web
# Opens browser to http://localhost:3000
```

**Database:**
- SurrealDB embedded, file-based
- Stored in `loaa/.data/db/`
- Gitignored

**MCP Server:**
```bash
cargo run -p loaa-mcp
# Listens on stdio for MCP protocol
```

### Future: Production Deployment

**Options to consider later:**
- **Home server**: Run on NAS or Raspberry Pi
- **VPS**: Small VPS with systemd service
- **Docker**: Containerize for easy deployment
- **Cloudflare Tunnels**: Expose local server securely

**Not needed for MVP** - just run locally on parent's laptop.

## Authentication Strategy (Phase 2)

**Approach**: Simple password-based auth, no OAuth complexity

**Implementation:**
- Password hashing with bcrypt or argon2
- Session cookies
- Server-side session store
- No JWT (unnecessary complexity)

**Roles:**
- Parent: full access to everything
- Child: read own data, mark tasks complete

## State Management

### Server State (Source of Truth)

All data lives in SurrealDB. Web server queries DB on demand.

### Client State (Leptos)

**Reactive signals** for UI state:
- Current user
- Selected kid filter
- Form inputs

**Server state** cached via Leptos resources:
- Tasks list
- Ledger entries
- Pending approvals

## Error Handling

**Strategy**: Type-safe errors with context

```rust
// In loaa-core
type Result<T> = anyhow::Result<T>;

// Add context at each layer
db::get_task(id)
    .context("Failed to fetch task")
    .context(format!("Task ID: {}", id))?;
```

**In Web UI**: Convert errors to user-friendly messages

**In MCP**: Return structured error responses per protocol

## Testing Strategy

### Unit Tests

- Core business logic in `loaa-core`
- Pure functions, easy to test
- No mocking needed for most logic

### Integration Tests

- Database operations
- Full workflow tests (create task → complete → check ledger)

### E2E Tests (Later)

- Playwright or similar
- Test full user workflows in browser
- Only add when UI stabilizes

**MVP**: Focus on core logic tests. UI tests can wait.

## Performance Considerations

**For MVP**: Premature optimization is evil. Focus on correctness.

**Expectations:**
- Single user (parent)
- ~10-50 tasks total
- ~1000 ledger entries/year
- Local database on laptop SSD

This is not a performance-sensitive application. Optimize only if we see issues.

## Security Considerations

### Phase 1 (Single User)

**Threat model**: None. Runs locally, parent only.

### Phase 2+ (Multi-User)

**Threats to consider:**
- Kids accessing each other's data
- Kids tampering with balances
- SQL injection (SurrealDB uses prepared queries)
- Session hijacking

**Mitigations:**
- Role-based access control
- Parameterized queries only
- HTTPOnly secure cookies
- CSRF tokens for mutations

**Not worried about:**
- DDoS or availability attacks
- Advanced persistent threats
- This is a family app, not a bank

## Monitoring & Observability

**Phase 1**: None. Just logs.

**Later**:
- Simple logging with `tracing`
- Optional metrics (task completions/day)
- Error tracking (sentry or similar)

## Code Style & Conventions

**Follow standard Rust conventions:**
- `rustfmt` for formatting
- `clippy` for linting
- Clear error messages with context
- Prefer `.context()` over `.expect()`
- Document public APIs

**Commit messages:**
- Imperative mood: "Add feature" not "Added feature"
- Under 50 characters
- See AGENTS.md for full guidelines

## Dependencies Philosophy

**Prefer:**
- Well-maintained crates
- Minimal dependency trees
- Pure Rust when possible
- Async-first (tokio ecosystem)

**Avoid:**
- Unmaintained crates
- Heavy dependencies for simple tasks
- Blocking I/O in async contexts

## Future Architecture Considerations

### If We Scale

**Multi-tenancy**: Each family gets their own database
**Real-time**: WebSocket updates for task completions
**Mobile**: Native apps or PWA
**Offline**: Local-first sync model

### If We Pivot

**From SurrealDB → SQLite:**
- Swap `loaa-core` database layer
- Use sqlx or diesel
- Minimal impact on web/mcp crates

**From Leptos → Axum + HTMX:**
- Keep `loaa-core` unchanged
- Rewrite `loaa-web` with templates
- Simpler, more traditional architecture

The crate structure makes these pivots possible.

## Open Questions

- SurrealDB query performance at scale?
- Leptos SSR deployment story?
- Best way to handle database migrations?
- How to backup/restore user data?
- Should we use WebSockets or polling?

We'll answer these through implementation and real usage.
