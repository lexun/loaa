# Tech Stack Decision

## Chosen Stack

- **Web Framework**: Leptos
- **Database**: SurrealDB
- **Architecture**: Workspace with core/web/mcp crates

## Rationale

### Leptos
- Full-stack Rust (server + client)
- Modern reactive UI with signals
- SSR with hydration
- Type safety across frontend/backend
- User preference: wanted to start with Leptos immediately

### SurrealDB
- Experimental choice to explore new technology
- Embedded mode for simple deployment
- Built-in graph relations (good for task prerequisites)
- Native Rust implementation
- User explicitly wanted to try it out

### Workspace Structure
- Follows user's preferred pattern (see nova project)
- Separation of concerns:
  - `core` - domain models and business logic
  - `web` - Leptos web application
  - `mcp` - MCP server for AI integration

## Alternatives Considered

- Axum + HTMX (simpler but user prefers modern reactive UI)
- SQLite (more mature but user wants to explore SurrealDB)
- Monolithic structure (less separation, harder to test)

## Future Considerations

- May need to evaluate SurrealDB's production readiness
- Backup and migration strategies for SurrealDB
- Deployment patterns for Leptos SSR
