# OAuth 2.1 with JWT Implementation Plan

## Overview

We're implementing OAuth 2.1 authentication for the MCP server using **JSON Web Tokens (JWT)** for stateless, scalable authentication.

## Architecture

### Components

1. **Web Server (Authorization Server)** - Port 3000
   - Issues JWT access tokens
   - Handles OAuth authorization flow
   - Users authenticate with username/password

2. **MCP Server (Resource Server)** - Port 3001
   - Validates JWT tokens
   - Provides MCP tools to Claude
   - No state required!

### Key Principle: Stateless Authentication

- **No shared state** between web and MCP servers
- **No database lookups** for token validation
- **JWT signature verification** is the only check needed
- Both servers share a **JWT signing secret** (via environment variable)

## OAuth Flow

```
1. User adds MCP in Claude
   └─> Claude redirects to: https://yourserver.com/oauth/authorize?client_id=...&code_challenge=...

2. User logs in at web server
   └─> Web server: POST /api/login (username=admin, password=admin123)
   └─> Creates session, redirects back to /oauth/authorize

3. Web server generates authorization code
   └─> Stores: code → {user_id, code_challenge, client_id, expiration}
   └─> Redirects to: https://claude.ai/api/mcp/auth_callback?code=XXX&state=YYY

4. Claude exchanges code for token
   └─> POST https://yourserver.com/oauth/token
   └─> Body: {code, code_verifier, client_id}
   └─> Web server validates PKCE (code_challenge == SHA256(code_verifier))
   └─> Returns: JWT access token

5. Claude calls MCP server
   └─> GET https://yourserver.com:3001/mcp
   └─> Header: Authorization: Bearer <JWT>
   └─> MCP server validates JWT signature
   └─> If valid, processes request
```

## JWT Structure

### Token Claims
```json
{
  "sub": "user_uuid",           // Subject (user ID)
  "iss": "https://yourserver.com",  // Issuer
  "aud": "loaa-mcp",            // Audience
  "exp": 1234567890,            // Expiration (Unix timestamp)
  "iat": 1234567890,            // Issued at
  "scope": "mcp:tools:read mcp:tools:write"
}
```

### Signing
- **Algorithm**: HS256 (HMAC-SHA256)
- **Secret**: Shared via `LOAA_JWT_SECRET` environment variable
- **Default**: Generate random secret on first run, save to `.env`

## State Management

### Authorization Codes (Temporary, In-Memory)
- **Storage**: HashMap in web server only
- **Lifetime**: 10 minutes
- **Purpose**: PKCE validation during code exchange
- **Cleanup**: Deleted after use or expiration

**Why in-memory is OK:**
- Codes are short-lived (10 min)
- Single-use only
- If server restarts, user just retries OAuth flow
- Not critical data

### Access Tokens (Stateless, No Storage)
- **Storage**: None! Token is self-contained
- **Validation**: JWT signature check only
- **Revocation**: Not supported in v1 (tokens expire after 24h)

## Implementation Steps

### 1. Update OAuth Module (web server)

**File**: `crates/web/src/oauth.rs`

Changes:
- Replace UUID tokens with JWT
- Add JWT signing on token creation
- Keep authorization codes in-memory (current approach is fine)

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,        // user_id
    iss: String,        // issuer (base URL)
    aud: String,        // audience ("loaa-mcp")
    exp: i64,           // expiration
    iat: i64,           // issued at
    scope: String,      // OAuth scopes
}

fn create_jwt(user_id: String, scope: String, secret: &str) -> Result<String> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        iss: "loaa".to_string(),
        aud: "loaa-mcp".to_string(),
        exp: now + 86400,  // 24 hours
        iat: now,
        scope,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}
```

### 2. Add JWT Validation Middleware (MCP server)

**File**: `crates/mcp/src/auth.rs` (new)

```rust
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn validate_jwt<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check Bearer scheme
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate JWT
    let secret = std::env::var("LOAA_JWT_SECRET")
        .expect("LOAA_JWT_SECRET must be set");

    let validation = Validation::default();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Token is valid, proceed
    Ok(next.run(req).await)
}
```

### 3. Update MCP Server to Use Middleware

**File**: `crates/mcp/src/main.rs`

```rust
async fn run_http_server(server: LoaaServer, config: &Config) -> Result<()> {
    // ... existing setup ...

    let app = Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn(auth::validate_jwt));  // <-- Add this

    // ... rest of setup ...
}
```

### 4. Add JWT Secret Management

**File**: `crates/core/src/config.rs`

```rust
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub jwt_secret: String,  // <-- Add this
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database: DatabaseConfig::from_env(),
            server: ServerConfig::from_env(),
            jwt_secret: std::env::var("LOAA_JWT_SECRET")
                .unwrap_or_else(|_| {
                    eprintln!("WARNING: LOAA_JWT_SECRET not set. Using insecure default.");
                    eprintln!("Generate a secret: openssl rand -base64 32");
                    "insecure-default-change-me".to_string()
                }),
        }
    }
}
```

### 5. Update Environment Variables

**File**: `.env.example`

```bash
# JWT signing secret (REQUIRED for production)
# Generate with: openssl rand -base64 32
LOAA_JWT_SECRET=your-secret-here

# Both web and MCP servers must use the same secret
```

## Deployment Modes

### Mode 1: All-in-One (Recommended for MVP)
Single binary runs web + MCP + embedded DB:

```bash
LOAA_DB_MODE=embedded
LOAA_DB_PATH=./data/loaa.db
LOAA_JWT_SECRET=<generate-random>
LOAA_INCLUDE_MCP=true
LOAA_MCP_PORT=3001
cargo run -p loaa-web
```

**Ports:**
- 3000: Web UI + OAuth endpoints
- 3001: MCP server

### Mode 2: Separate Processes (For Scaling)
Web and MCP run separately, share JWT secret:

```bash
# Terminal 1: Web server
LOAA_DB_MODE=embedded
LOAA_DB_PATH=./data/loaa.db
LOAA_JWT_SECRET=same-secret-here
cargo run -p loaa-web

# Terminal 2: MCP server
LOAA_DB_MODE=embedded
LOAA_DB_PATH=./data/loaa.db
LOAA_JWT_SECRET=same-secret-here
LOAA_MCP_TRANSPORT=http
cargo run -p loaa-mcp
```

### Mode 3: Docker (Production)
Both containers share JWT secret via environment:

```yaml
services:
  web:
    image: loaa-web
    environment:
      - LOAA_JWT_SECRET=${LOAA_JWT_SECRET}
    ports:
      - "3000:3000"

  mcp:
    image: loaa-mcp
    environment:
      - LOAA_JWT_SECRET=${LOAA_JWT_SECRET}
    ports:
      - "3001:3001"
```

## Security Considerations

### JWT Secret
- **MUST** be random and high-entropy
- **NEVER** commit to git
- **ROTATE** periodically (invalidates all tokens)
- **LENGTH**: At least 32 bytes (256 bits)

```bash
# Generate secure secret
openssl rand -base64 32
```

### Token Expiration
- Access tokens expire after 24 hours
- No refresh tokens in v1 (user re-authenticates)
- Future: Add refresh token support

### HTTPS in Production
- **REQUIRED** for production deployments
- Prevents token theft via man-in-the-middle
- Use Let's Encrypt for free certificates

## Testing Plan

### 1. Local Testing
```bash
# Start web server
LOAA_JWT_SECRET=test-secret-12345 cargo leptos watch

# In another terminal, test token generation
curl http://localhost:3000/.well-known/oauth-authorization-server
```

### 2. OAuth Flow Testing
1. Add MCP in Claude Desktop
2. URL: `http://localhost:3000`
3. Login with admin/admin123
4. Verify token is issued
5. Call MCP tool, verify it works

### 3. JWT Validation Testing
```bash
# Generate a JWT manually
# Call MCP endpoint with valid token (should work)
# Call MCP endpoint with invalid token (should 401)
# Call MCP endpoint with expired token (should 401)
```

## Future Enhancements

### Token Revocation
- Add token blacklist in database
- Check blacklist on each request
- Allow admin to revoke tokens

### Refresh Tokens
- Issue long-lived refresh tokens
- Allow obtaining new access tokens without re-authentication
- Store refresh tokens in database

### Scopes
- Implement granular permissions
- `mcp:tasks:read`, `mcp:tasks:write`, etc.
- Validate scopes on each MCP call

## Migration Path

If we ever need to migrate from in-memory authorization codes:

1. Add database table for authorization codes
2. No changes to JWT (already stateless)
3. Backwards compatible - just more reliable

## References

- [RFC 6749 - OAuth 2.0](https://datatracker.ietf.org/doc/html/rfc6749)
- [RFC 7519 - JSON Web Token (JWT)](https://datatracker.ietf.org/doc/html/rfc7519)
- [RFC 7636 - PKCE](https://datatracker.ietf.org/doc/html/rfc7636)
- [MCP OAuth Spec](https://modelcontextprotocol.io/specification/draft/basic/authorization)
