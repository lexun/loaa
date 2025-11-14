# Running Loa'a Locally

## Prerequisites

- Nix with devenv (for automatic dependency management)
- Or manually: Rust, SurrealDB 2.3+, just

**Note**: The Rust client SDK (v2.3) requires SurrealDB server v2.3 or higher.

## Quick Start

### 1. Enter the development shell (if using devenv)

```bash
devenv shell
```

### 2. Start SurrealDB server

```bash
just db
```

This starts SurrealDB on `127.0.0.1:8000` with:
- Username: `root`
- Password: `root`
- Database file: `data/loaa.db`

**Keep this terminal open** - the database server needs to stay running.

### 3. Run the MCP server (in a new terminal)

```bash
just mcp
```

The MCP server will connect to SurrealDB and expose tools for Claude Code.

### 4. Or run the web server (in a new terminal)

```bash
just web
```

The web server will connect to SurrealDB and serve the UI.

## Running Both Servers Simultaneously

You can now run both the MCP server and web server at the same time! They both connect to the same SurrealDB instance via WebSocket.

**Terminal 1:**
```bash
just db
```

**Terminal 2:**
```bash
just mcp
```

**Terminal 3:**
```bash
just web
```

Changes made through the MCP server (via Claude Code) will be immediately visible in the web UI and vice versa.

## Database Management

### Clean the database (delete all data)

```bash
just clean
```

### Seed with test data

```bash
just seed
```

### Inspect the database

While SurrealDB is running, you can connect with the CLI:

```bash
surreal sql --endpoint http://127.0.0.1:8000 --username root --password root --namespace loaa --database main
```

## Troubleshooting

### "Resource temporarily unavailable" error

This means the database file is locked. Make sure:
1. The SurrealDB server is running (`just db`)
2. No other processes have the database file open

### MCP server can't connect

Make sure SurrealDB is running first (`just db`). The MCP server expects to connect to `127.0.0.1:8000`.

### Web server can't connect

Same as above - ensure SurrealDB is running on port 8000.
