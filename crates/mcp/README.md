# Loa'a MCP Server

Model Context Protocol (MCP) server for the Loa'a chore tracking system.

## Overview

This MCP server exposes Loa'a's core functionality through natural language commands via Claude Code or other MCP clients. It provides tools for managing kids, tasks, ledgers, and task completions.

## Available Tools

### Kid Management
- **create_kid** - Create a new kid in the system
- **list_kids** - List all kids

### Task Management
- **create_task** - Create a new task with value and cadence
- **list_tasks** - List all tasks
- **update_task** - Update an existing task
- **delete_task** - Delete a task by ID

### Task Completion & Ledger
- **complete_task** - Mark a task as complete for a kid (creates ledger entry, resets recurring tasks)
- **get_ledger** - Get transaction history and balance for a kid
- **adjust_balance** - Manually adjust a kid's balance

## Configuration

### Claude Code

Add this to your Claude Code MCP configuration:

```json
{
  "mcpServers": {
    "loaa": {
      "command": "cargo",
      "args": ["run", "-p", "loaa-mcp"],
      "env": {
        "LOAA_DB_PATH": "./data/loaa.db"
      }
    }
  }
}
```

### Environment Variables

- `LOAA_DB_PATH` - Path to the SurrealDB database file (default: `../../data/loaa.db` relative to manifest)

## Usage Examples

Once configured with Claude Code, you can use natural language commands like:

- "Create a new kid named Alice"
- "Create a task called 'Take out trash' worth $1.50 daily"
- "List all tasks"
- "Alice finished taking out the trash"
- "How much do we owe each kid?"
- "Add $5 to Bob's balance for extra chores"

## Development

### Running Locally

```bash
# From the project root
cargo run -p loaa-mcp

# With custom database path
LOAA_DB_PATH=/path/to/loaa.db cargo run -p loaa-mcp
```

### Building

```bash
cargo build -p loaa-mcp --release
```

## Architecture

The MCP server uses:
- **rmcp** (v0.8.5) - Official Rust SDK for Model Context Protocol
- **stdio transport** - Communicates via standard input/output
- **schemars** - Automatic JSON schema generation for tool parameters
- **tokio** - Async runtime for handling requests

All business logic is delegated to `loaa-core` crate, keeping the MCP server as a thin protocol adapter.

## Error Handling

The server uses MCP standard error types:
- `invalid_request` - Invalid parameters or validation errors
- `resource_not_found` - Entity (task/kid) not found
- `internal_error` - Database or workflow errors

## Transport

This server uses stdio transport, communicating over standard input/output using JSON-RPC protocol. This makes it compatible with Claude Code and other MCP clients that support stdio.
