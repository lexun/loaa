# Running Loa'a Locally

## Prerequisites

- Nix with devenv (for automatic dependency management)
- Or manually: Rust, SurrealDB 2.3+, just

**Note**: The Rust client SDK (v2.3) requires SurrealDB server v2.3 or higher.

## Quick Start (Recommended)

### Start all services

```bash
# Foreground mode (default) - logs shown in terminal
just start

# Background mode - services run as daemon
just start -d
```

This uses process-compose to start and manage all services:
- **SurrealDB** database server on port 8000
- **Web server** on port 3000 (waits for database to be healthy)

Features:
- ✅ **Single command** - no more managing multiple terminals
- ✅ **Automatic dependencies** - web server waits for database to be ready
- ✅ **Health checks** - ensures services are actually running
- ✅ **Auto-restart** - services restart on failure (up to 5 attempts)
- ✅ **Clean shutdown** - easy stop/restart commands

### Background vs Foreground

**Foreground mode (`just start`):**
- Interactive TUI (Terminal User Interface)
- Navigate logs, view process status
- Ctrl+C stops all services
- Best for active development
- If services already running in background, attaches to them with TUI

**Background mode (`just start -d`):**
- Services run as daemon (no TUI)
- Terminal freed for other work
- Run `just start` again to attach with TUI
- Use `just logs` to tail log file
- Use `just stop` to stop services

### Stop all services

```bash
just stop
```

Stops all services cleanly, whether running in foreground or background.

### Restart all services

```bash
# Restart in foreground
just restart

# Restart in background
just restart -d
```

### View service logs

```bash
# View all service logs (combined)
just logs

# View logs for a specific service
just log db    # Database logs only
just log web   # Web server logs only
```

Logs are automatically captured when running in background mode.

## Alternative: Manual Service Management

If you prefer to manage services individually in separate terminals:

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

You can run both the MCP server and web server at the same time! They both connect to the same SurrealDB instance via WebSocket.

**Using devenv up:**
```bash
devenv up
```

**Or manually with multiple terminals:**

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

### Reset database (clean + seed)

```bash
just reset
```

This removes all data and re-seeds with the default test data (Auri, Zevi, Yasu + 8 tasks). Perfect for getting back to a known state.

### Seed with test data

```bash
just seed
```

Adds the default test data without cleaning existing data.

### Clean the database (delete all data)

```bash
just clean
```

WARNING: This deletes all data permanently!

### Inspect the database

While SurrealDB is running, you can connect with the CLI:

```bash
surreal sql --endpoint http://127.0.0.1:8000 --username root --password root --namespace loaa --database main
```

## Troubleshooting

### Port already in use (EADDRINUSE)

If you get an error that port 8000 or 3000 is already in use:

**Solution 1: Use `devenv up` (recommended)**
- This prevents port conflicts by managing all services in one place
- Stopping services with Ctrl+C ensures clean shutdown

**Solution 2: Find and kill the process**
```bash
# Find process using port 8000 (SurrealDB)
lsof -i :8000

# Find process using port 3000 (web server)
lsof -i :3000

# Kill the process
kill <PID>
```

**Solution 3: Stop all related processes**
```bash
# Kill all SurrealDB processes
pkill -f surreal

# Kill all web server processes
pkill -f simple_server
```

### "Resource temporarily unavailable" error

This means the database file is locked. Make sure:
1. The SurrealDB server is running (`devenv up` or `just db`)
2. No other processes have the database file open
3. If using manual management, only one SurrealDB instance is running

### MCP server can't connect

Make sure SurrealDB is running first. The MCP server expects to connect to `127.0.0.1:8000`.

**Using devenv:**
```bash
devenv up  # Starts database automatically
```

**Manual approach:**
```bash
just db  # Start database first
just mcp # Then start MCP in another terminal
```

### Web server can't connect

Same as above - ensure SurrealDB is running on port 8000.

### `devenv up` fails to start

1. Make sure you're in the devenv shell: `devenv shell`
2. Check if ports are already in use (see "Port already in use" above)
3. Try stopping all services first: `pkill -f surreal && pkill -f simple_server`
4. Review logs in the TUI for specific error messages
