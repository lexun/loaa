# Loa'a Deployment Guide

This guide covers how to deploy Loa'a for production use, including OAuth-authenticated MCP server access for Claude.

## Prerequisites

- Docker and Docker Compose (recommended)
- OR Rust toolchain with cargo-leptos
- A publicly accessible server or domain (for OAuth callback)
- (Optional) Reverse proxy with SSL/TLS certificate

## Quick Start with Docker Compose

### 1. Generate a JWT Secret

```bash
openssl rand -base64 32
```

### 2. Create .env File

```bash
cp .env.example .env
```

Edit `.env` and set:
```bash
LOAA_JWT_SECRET=<your-generated-secret>
LOAA_BASE_URL=https://your-domain.com  # Or http://your-ip:3000 for testing
```

### 3. Run with Docker Compose

```bash
docker-compose up -d
```

This will:
- Build the Loa'a image
- Start the web server on port 3000
- Start the MCP server on port 3001
- Use an embedded database (persisted in Docker volume)

### 4. Access the Application

- Web UI: `http://localhost:3000`
- MCP endpoint: `http://localhost:3001/mcp`
- OAuth discovery: `http://localhost:3000/.well-known/oauth-authorization-server`

Default credentials:
- Username: `admin`
- Password: `admin123`

**IMPORTANT:** Change these credentials immediately in production!

## Deployment Modes

### All-in-One Mode (Recommended)

Single process runs both web server and MCP server:

```bash
LOAA_INCLUDE_MCP=true \
LOAA_DB_MODE=embedded \
LOAA_DB_PATH=./data/loaa.db \
LOAA_JWT_SECRET=<your-secret> \
LOAA_BASE_URL=https://your-domain.com \
./loaa-web
```

**Ports:**
- 3000: Web UI + OAuth endpoints
- 3001: MCP server

### Separate Processes Mode

For horizontal scaling, run web and MCP servers separately:

**Web Server:**
```bash
LOAA_DB_MODE=embedded \
LOAA_DB_PATH=./data/loaa.db \
LOAA_JWT_SECRET=<your-secret> \
LOAA_BASE_URL=https://your-domain.com \
./loaa-web
```

**MCP Server (in another terminal/process):**
```bash
LOAA_DB_MODE=embedded \
LOAA_DB_PATH=./data/loaa.db \
LOAA_JWT_SECRET=<your-secret> \
LOAA_BASE_URL=https://your-domain.com \
LOAA_MCP_TRANSPORT=http \
LOAA_MCP_PORT=3001 \
./loaa-mcp
```

**IMPORTANT:** Both servers MUST use the same `LOAA_JWT_SECRET`.

## Production Configuration

### Reverse Proxy with Nginx

Example Nginx configuration for SSL termination:

```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    # Web UI and OAuth endpoints
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # MCP server endpoint
    location /mcp {
        proxy_pass http://localhost:3001/mcp;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support for MCP
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### Environment Variables Reference

**Database:**
- `LOAA_DB_MODE`: `memory`, `embedded`, or `remote` (default: `memory`)
- `LOAA_DB_PATH`: Path for embedded database (e.g., `/app/data/loaa.db`)
- `LOAA_DB_URL`: URL for remote SurrealDB (e.g., `127.0.0.1:8000`)

**Server:**
- `LOAA_HOST`: Server host (default: `127.0.0.1`)
- `LOAA_PORT`: Web server port (default: `3000`)
- `LOAA_INCLUDE_MCP`: Run MCP in same process (default: `false`)

**MCP:**
- `LOAA_MCP_HOST`: MCP server host (default: `127.0.0.1`)
- `LOAA_MCP_PORT`: MCP server port (default: `3001`)
- `LOAA_MCP_TRANSPORT`: `stdio` or `http` (default: `stdio`)

**OAuth:**
- `LOAA_JWT_SECRET`: **REQUIRED** - JWT signing secret (use `openssl rand -base64 32`)
- `LOAA_BASE_URL`: Base URL for OAuth redirects (e.g., `https://your-domain.com`)

## Connecting Claude to Your MCP Server

### 1. In Claude Desktop/Mobile

Add a custom MCP connector:
- **Name:** Loa'a
- **Remote MCP server URL:** `https://your-domain.com/mcp`

### 2. OAuth Authorization Flow

When you add the connector, Claude will:
1. Redirect you to your server's login page
2. Ask you to login (default: admin/admin123)
3. Request authorization
4. Exchange authorization code for JWT access token
5. Use the JWT for all subsequent MCP requests

### 3. Verify Connection

After authorization, try asking Claude:
- "List all kids in the system"
- "Create a new task for taking out the trash worth $1.50"

## Security Considerations

### JWT Secret Management

- ✅ Generate with: `openssl rand -base64 32`
- ✅ Store in environment variables or secrets manager
- ✅ Rotate periodically (invalidates all tokens)
- ❌ Never commit to git
- ❌ Never use the default value in production

### HTTPS Requirements

**REQUIRED** for production deployments:
- Prevents token theft via man-in-the-middle attacks
- Required for OAuth security
- Use Let's Encrypt for free SSL certificates

### Database Backups

For embedded mode, backup the database file:

```bash
# Stop the application first
docker-compose down

# Backup
cp /path/to/loaa.db /backups/loaa-$(date +%Y%m%d).db

# Restart
docker-compose up -d
```

## Troubleshooting

### OAuth Flow Fails

Check that:
- `LOAA_BASE_URL` matches your actual domain/IP
- `LOAA_JWT_SECRET` is set on both web and MCP servers
- Claude's redirect URI is whitelisted (should work by default)

### MCP Server Not Accessible

Verify:
- Port 3001 is exposed and accessible
- JWT authentication is working (check logs)
- Firewall allows incoming connections on port 3001

### Database Issues

For embedded mode:
- Ensure `LOAA_DB_PATH` directory exists and is writable
- Check disk space
- Verify file permissions

## Monitoring

### Health Checks

- Web server: `GET /.well-known/oauth-authorization-server`
- MCP server: Requires JWT authentication

### Logs

View logs:
```bash
docker-compose logs -f loaa
```

Filter for errors:
```bash
docker-compose logs loaa | grep -i error
```

## Updating

### Pull Latest Image

```bash
docker-compose pull
docker-compose up -d
```

### Build from Source

```bash
docker-compose build
docker-compose up -d
```

## Support

For issues, questions, or feature requests, please open an issue on GitHub.
