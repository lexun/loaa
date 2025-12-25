# Loa'a

A chore and rewards tracking system that helps kids stay motivated through gamified task completion with monetary rewards.

## Project Status

🚧 **Early Development** - Initial repository setup complete, beginning implementation.

## Vision

Loa'a (Hawaiian for "to earn" or "receive") helps children:
- See what tasks need to be done
- Understand the value of their work
- Track their earnings in a personal ledger
- Redeem rewards (screen time, cash, privileges)

Key features (planned):
- Task definitions with monetary values and refresh cadence
- Completion workflow (kids mark done → parent approves)
- Ledger system tracking earnings and spending
- Prerequisites/unlocks (complete X to unlock Y)
- Dynamic bounties (value increases over time)
- MCP server for AI assistant integration

## Tech Stack

- **Language**: Rust
- **Web Framework**: Leptos (full-stack SSR)
- **Database**: SurrealDB (embedded mode)
- **Architecture**: Workspace with core/web/mcp crates

## Development

This project uses [devenv](https://devenv.sh) with direnv for automatic environment activation.

### Prerequisites

- nix
- direnv
- devenv

### Setup

```bash
# Allow direnv to load the environment
direnv allow

# Build the project
cargo build

# Run the web server (when implemented)
cargo run -p loaa-web

# Run the MCP server (when implemented)
cargo run -p loaa-mcp
```

## Project Structure

```
loaa/
├── crates/
│   ├── core/       # Domain models and business logic
│   ├── web/        # Leptos web application
│   └── mcp/        # MCP server for AI integration
└── docs/           # Design decisions and documentation
```

See [AGENTS.md](AGENTS.md) for development workflow details.

## License

This project is licensed under the [Elastic License 2.0](LICENSE). You are free to use, modify, and distribute this software, but you may not provide it to others as a hosted or managed service.
