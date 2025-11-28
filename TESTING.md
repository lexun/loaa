# OAuth Testing Guide

This guide walks you through testing the complete OAuth 2.1 flow with JWT authentication.

## Prerequisites

- Server running locally on port 3000 (web) and 3001 (MCP)
- curl (for command-line testing)
- Claude Desktop or Claude Mobile (for real MCP integration testing)

## Part 1: Manual OAuth Flow Testing (Without Claude)

### Step 1: Verify OAuth Discovery Endpoint

```bash
curl http://localhost:3000/.well-known/oauth-authorization-server | jq
```

**Expected output:**
```json
{
  "issuer": "http://127.0.0.1:3000",
  "authorization_endpoint": "http://127.0.0.1:3000/oauth/authorize",
  "token_endpoint": "http://127.0.0.1:3000/oauth/token",
  "code_challenge_methods_supported": ["S256"],
  "grant_types_supported": ["authorization_code"],
  "response_types_supported": ["code"],
  "scopes_supported": ["mcp:tools:read", "mcp:tools:write"]
}
```

### Step 2: Generate PKCE Challenge

```bash
# Generate code verifier (random string)
CODE_VERIFIER=$(openssl rand -base64 32 | tr -d '=' | tr '+/' '-_')
echo "Code Verifier: $CODE_VERIFIER"

# Generate code challenge (SHA256 hash)
CODE_CHALLENGE=$(echo -n "$CODE_VERIFIER" | openssl dgst -binary -sha256 | base64 | tr -d '=' | tr '+/' '-_')
echo "Code Challenge: $CODE_CHALLENGE"
```

### Step 3: Start Authorization Flow

Open this URL in your browser (replace CODE_CHALLENGE with the value from step 2):

```
http://localhost:3000/oauth/authorize?client_id=test-client&redirect_uri=http://localhost:8080/callback&scope=mcp:tools:read%20mcp:tools:write&state=random-state-123&code_challenge=CODE_CHALLENGE&code_challenge_method=S256
```

**What should happen:**
1. You'll be redirected to the login page
2. Login with: `admin` / `admin123`
3. You'll be redirected back to `http://localhost:8080/callback?code=XXX&state=random-state-123`
4. Copy the `code=XXX` value from the URL

### Step 4: Exchange Authorization Code for JWT

```bash
# Replace AUTHORIZATION_CODE with the code from step 3
# Replace CODE_VERIFIER with the value from step 2

curl -X POST http://localhost:3000/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=AUTHORIZATION_CODE" \
  -d "client_id=test-client" \
  -d "redirect_uri=http://localhost:8080/callback" \
  -d "code_verifier=$CODE_VERIFIER" | jq
```

**Expected output:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### Step 5: Decode the JWT (Optional)

Visit https://jwt.io and paste the access token to see the decoded claims:

```json
{
  "sub": "user-uuid-here",
  "iss": "http://127.0.0.1:3000",
  "aud": "loaa-mcp",
  "exp": 1234567890,
  "iat": 1234567890,
  "scope": "mcp:tools:read mcp:tools:write"
}
```

### Step 6: Test MCP Endpoint Without Token (Should Fail)

```bash
curl http://localhost:3001/mcp
```

**Expected:** `401 Unauthorized`

### Step 7: Test MCP Endpoint With JWT Token (Should Work)

```bash
# Replace ACCESS_TOKEN with the token from step 4

curl -H "Authorization: Bearer ACCESS_TOKEN" \
     http://localhost:3001/mcp
```

**Expected:** Should return MCP server response (not 401)

---

## Part 2: Testing with Claude Desktop/Mobile

### Step 1: Open Claude Desktop or Mobile App

Navigate to Settings → MCP Connectors (or Custom Connectors)

### Step 2: Add Custom MCP Connector

Fill in the form:
- **Name:** Loa'a (or any name you prefer)
- **Remote MCP server URL:** `http://localhost:3001/mcp`
- Leave OAuth fields empty (will auto-discover)

### Step 3: Authorize the Connection

Claude will:
1. Discover OAuth endpoints via `/.well-known/oauth-authorization-server`
2. Redirect you to `http://localhost:3000/oauth/authorize`
3. Show you the login page

**Login with:**
- Username: `admin`
- Password: `admin123`

### Step 4: Complete Authorization

After login:
1. You'll see an authorization consent screen (auto-approved in current implementation)
2. Claude will exchange the code for a JWT token
3. You'll be redirected back to Claude

### Step 5: Test MCP Tools in Claude

Try these commands in Claude:

```
1. "List all kids in the system"
2. "Create a new kid named Alice"
3. "Create a task for taking out the trash worth $2.50"
4. "List all tasks"
```

**Expected behavior:**
- Claude should successfully call the MCP tools
- You should see responses with task/kid data
- All requests should include the JWT Bearer token

---

## Part 3: Debugging Failed OAuth Flow

### Check Server Logs

```bash
tail -f /Users/luke/workspace/lexun/loaa/.devenv/processes.log | grep -E "(OAuth|JWT|MCP|error)"
```

### Common Issues

#### 1. "Invalid authorization code"

**Cause:** Code was already used or expired (10 minute lifetime)

**Fix:** Start a new authorization flow from Step 3

#### 2. "Invalid code verifier"

**Cause:** PKCE code_challenge doesn't match code_verifier

**Fix:** Make sure you're using the same CODE_VERIFIER that generated CODE_CHALLENGE

#### 3. "JWT validation failed"

**Cause:** Token signature invalid or JWT_SECRET mismatch

**Fix:** Ensure both web and MCP servers have the same `LOAA_JWT_SECRET`

```bash
# Check environment variable
echo $LOAA_JWT_SECRET
```

#### 4. "401 Unauthorized" on MCP endpoint

**Cause:** Missing or invalid Bearer token

**Fix:**
- Check Authorization header format: `Bearer <token>`
- Verify token hasn't expired (24 hour lifetime)
- Check JWT_SECRET matches between servers

#### 5. Connection refused on port 3001

**Cause:** MCP server not running

**Fix:** Verify `LOAA_INCLUDE_MCP=true` in environment:

```bash
tail -f /Users/luke/workspace/lexun/loaa/.devenv/processes.log | grep "MCP"
```

Should see:
```
📦 All-in-one mode: MCP server will be started
🚀 Starting embedded MCP server...
📡 MCP server will listen on http://127.0.0.1:3001
```

---

## Part 4: Testing Token Expiration

### Generate and Wait

1. Get a JWT token using steps 2-4 above
2. Wait 24 hours (or modify code to use shorter expiration for testing)
3. Try using the token - should get 401 Unauthorized

### Manual Expiration Test

You can modify `crates/web/src/oauth.rs` temporarily:

```rust
// Change from 24 hours to 10 seconds
exp: now + 10,  // Instead of: now + 86400
```

Then rebuild and test token expiration after 10 seconds.

---

## Part 5: Security Verification

### Verify PKCE Protection

Try exchanging code without correct verifier:

```bash
curl -X POST http://localhost:3000/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=AUTHORIZATION_CODE" \
  -d "client_id=test-client" \
  -d "redirect_uri=http://localhost:8080/callback" \
  -d "code_verifier=wrong-verifier"
```

**Expected:** Error about invalid code verifier

### Verify JWT Signature

Try using a tampered token:

```bash
# Take a valid JWT and change one character
TAMPERED_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.TAMPERED.signature"

curl -H "Authorization: Bearer $TAMPERED_TOKEN" \
     http://localhost:3001/mcp
```

**Expected:** 401 Unauthorized (JWT validation failed)

---

## Checklist

- [ ] OAuth discovery endpoint returns correct metadata
- [ ] Authorization flow redirects to login
- [ ] Login with admin/admin123 works
- [ ] Authorization code is generated
- [ ] Token exchange returns valid JWT
- [ ] JWT contains correct claims (sub, iss, aud, exp, iat, scope)
- [ ] MCP endpoint rejects requests without token
- [ ] MCP endpoint accepts requests with valid JWT
- [ ] Claude Desktop/Mobile can connect via OAuth
- [ ] Claude can call MCP tools successfully
- [ ] Expired tokens are rejected
- [ ] Tampered tokens are rejected
- [ ] PKCE protects against code interception

---

## Next Steps

Once all tests pass:
1. Update default credentials (admin/admin123)
2. Generate a secure JWT secret: `openssl rand -base64 32`
3. Set up HTTPS for production
4. Deploy using Docker or your preferred method

See `DEPLOYMENT.md` for production deployment instructions.
