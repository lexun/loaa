# Deployment Configuration

Loa'a supports flexible deployment modes through environment-based configuration. You can run everything locally with an embedded database, or deploy remotely with separate services.

## Environment Variables

### Database Configuration

**`LOAA_DB_MODE`** - Database connection mode
- `memory` - In-memory database (development only, data lost on restart)
- `embedded` - File-based RocksDB (persistent, single instance)
- `remote` - WebSocket connection to separate SurrealDB server (default: remote)

**`LOAA_DB_PATH`** - Database file path (required for embedded mode)
- Example: `./data/loaa.db` or `/var/lib/loaa/db`

**`LOAA_DB_URL`** - Database URL (required for remote mode)
- Example: `127.0.0.1:8000` or `db.example.com:8000`

### Web Server Configuration

**`LOAA_HOST`** - Web server host (default: `127.0.0.1`)
**`LOAA_PORT`** - Web server port (default: `3000`)

### MCP Server Configuration

**`LOAA_MCP_TRANSPORT`** - MCP server transport mode
- `stdio` - Standard input/output for local Claude Desktop/Code (default)
- `http` - HTTP transport for remote access

**`LOAA_MCP_HOST`** - MCP HTTP server host (default: `127.0.0.1`)
**`LOAA_MCP_PORT`** - MCP HTTP server port (default: `3001`)

## Deployment Scenarios

### 1. Local Development (Default)

Everything runs locally with an external SurrealDB server:

```bash
# Terminal 1: Start SurrealDB
surreal start --log info --user root --pass root --bind 127.0.0.1:8000 memory

# Terminal 2: Start web server (uses remote DB by default)
just start

# Terminal 3: Use MCP server (stdio mode by default)
# Configure Claude Desktop to use: cargo run -p loaa-mcp
```

### 2. Local with Embedded Database

Single instance with persistent file-based storage:

```bash
# Set environment variables
export LOAA_DB_MODE=embedded
export LOAA_DB_PATH=./data/loaa.db

# Start web server
cd crates/web && cargo leptos watch

# Start MCP server
cargo run -p loaa-mcp
```

**Pros:**
- No separate database process
- Data persists to disk
- Simpler setup

**Cons:**
- Cannot scale horizontally
- Database locked to single process

### 3. All-in-One Server

Web server + MCP server + embedded database in a single process:

```bash
export LOAA_DB_MODE=embedded
export LOAA_DB_PATH=./data/loaa.db
export LOAA_INCLUDE_MCP=true
export LOAA_MCP_TRANSPORT=http

# Start combined server
cargo run -p loaa-web
```

**Use case:** Simple deployment to a single VPS or container

### 4. Remote MCP Server (Production)

Separate web server and remote MCP server with shared database:

```bash
# Shared config
export LOAA_DB_MODE=remote
export LOAA_DB_URL=db.example.com:8000

# Server 1: Web application
cd crates/web && cargo leptos watch

# Server 2: MCP HTTP server (accessible remotely)
export LOAA_MCP_TRANSPORT=http
export LOAA_MCP_HOST=0.0.0.0  # Listen on all interfaces
export LOAA_MCP_PORT=3001
cargo run -p loaa-mcp

# Configure Claude Desktop/Mobile with:
# http://your-server.com:3001/mcp
```

**Pros:**
- Scalable (can run multiple instances)
- Access from anywhere
- Claude Desktop, mobile, and Code all connect to same server

**Cons:**
- Requires authentication (implement OAuth)
- More complex deployment

### 5. Docker Compose

All services in containers:

```yaml
version: '3.8'

services:
  db:
    image: surrealdb/surrealdb:latest
    command: start --log info --user root --pass root --bind 0.0.0.0:8000 memory
    ports:
      - "8000:8000"

  web:
    build: .
    environment:
      - LOAA_DB_MODE=remote
      - LOAA_DB_URL=db:8000
    ports:
      - "3000:3000"
    depends_on:
      - db

  mcp:
    build: .
    environment:
      - LOAA_DB_MODE=remote
      - LOAA_DB_URL=db:8000
      - LOAA_MCP_TRANSPORT=http
      - LOAA_MCP_HOST=0.0.0.0
      - LOAA_MCP_PORT=3001
    ports:
      - "3001:3001"
    depends_on:
      - db
```

## Deployment Platforms

### Fly.io

Best for: Global edge deployment with low latency

```toml
# fly.toml
[env]
  LOAA_DB_MODE = "embedded"
  LOAA_DB_PATH = "/data/loaa.db"
  LOAA_MCP_TRANSPORT = "http"
  LOAA_MCP_HOST = "0.0.0.0"

[mounts]
  source = "loaa_data"
  destination = "/data"
```

**Cost:** ~$5-10/month for small instance with persistent volume

### Shuttle.dev

Best for: Rust-native deployment with zero config

```rust
// Shuttle auto-provisions database and handles deployment
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Shuttle provides DATABASE_URL automatically
    std::env::set_var("LOAA_DB_MODE", "remote");
    // ... rest of your setup
}
```

**Cost:** Free tier available, then ~$20/month

### DigitalOcean Droplet

Best for: Maximum control and predictable pricing

```bash
# On your $5/month droplet
curl -sSf https://install.surrealdb.com | sh
cargo install cargo-leptos

export LOAA_DB_MODE=embedded
export LOAA_DB_PATH=/var/lib/loaa/db

# Set up systemd services for web and MCP
```

**Cost:** $5-10/month for droplet

### Render

Best for: Free tier for testing

```yaml
# render.yaml
services:
  - type: web
    name: loaa-web
    env: docker
    envVars:
      - key: LOAA_DB_MODE
        value: embedded
      - key: LOAA_DB_PATH
        value: /data/loaa.db
    disk:
      name: loaa-data
      mountPath: /data
      sizeGB: 1
```

**Cost:** Free tier (with cold starts), $7/month for always-on

## Security Considerations

### Before Remote Deployment

⚠️ **IMPORTANT:** The current implementation has NO authentication. Before deploying remotely:

1. **Implement authentication** (see `loaa-pzn` and `loaa-iu5` issues)
   - Basic: Username/password for web app
   - Advanced: OAuth 2.0 for MCP server

2. **Use HTTPS/TLS**
   - Let's Encrypt for SSL certificates
   - Reverse proxy (Caddy, nginx)

3. **Firewall rules**
   - Only expose necessary ports
   - Consider VPN for MCP access

4. **Environment secrets**
   - Never commit `.env` files
   - Use platform secret management

## Testing Different Modes

### Test Embedded Database

```bash
# Create test data
LOAA_DB_MODE=embedded LOAA_DB_PATH=./data/test.db cargo run --example test_embedded_db

# Verify persistence
LOAA_DB_MODE=embedded LOAA_DB_PATH=./data/test.db cargo run --example test_embedded_db
# Should show accumulated data
```

### Test HTTP MCP Transport

```bash
# Start server
LOAA_DB_MODE=embedded LOAA_DB_PATH=./data/loaa.db LOAA_MCP_TRANSPORT=http cargo run -p loaa-mcp

# Test endpoint (in another terminal)
curl -X POST http://127.0.0.1:3001/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream"
# Should return server info
```

### Test stdio MCP Transport

```bash
# Configure Claude Desktop's config.json
{
  "mcpServers": {
    "loaa": {
      "command": "cargo",
      "args": ["run", "-p", "loaa-mcp"],
      "env": {
        "LOAA_DB_MODE": "embedded",
        "LOAA_DB_PATH": "/absolute/path/to/data/loaa.db"
      }
    }
  }
}
```

## Migration Path

### Development → Production

1. **Start local:** Use default remote DB mode
2. **Test embedded:** Switch to `LOAA_DB_MODE=embedded` for standalone testing
3. **Add HTTP MCP:** Set `LOAA_MCP_TRANSPORT=http` for remote access testing
4. **Deploy:** Choose platform and add authentication
5. **Scale:** Move to separate DB when needed

## Troubleshooting

### Database Locked

```
Error: Resource temporarily unavailable
```

**Solution:** RocksDB locks the database file. Ensure no other process is using it.

### Port Already in Use

```
Error: Address already in use
```

**Solution:** Change port or kill existing process:
```bash
lsof -ti:3001 | xargs kill -9
```

### Cannot Connect to Remote DB

```
Error: Failed to connect to database
```

**Solution:** Check URL, ensure SurrealDB is running, verify firewall rules.

## Next Steps

- Implement authentication (issues `loaa-pzn`, `loaa-iu5`)
- Add health check endpoints
- Set up monitoring and logging
- Create Docker images
- Write deployment scripts for each platform
