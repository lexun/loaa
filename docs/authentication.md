# Loa'a Authentication

## Overview

Loa'a uses a hybrid authentication system:
- **Admin user**: Environment variable-based (no database record)
- **Regular users**: Database-backed (for future multi-user support)

## Admin Authentication

### How It Works

1. **Username**: Hardcoded as `admin`
2. **Password**: Stored in `LOAA_ADMIN_PASSWORD` environment variable
3. **No database record**: Admin exists only in code
4. **Session**: Stores `user_id = "admin"` in session

### Password Management

The admin password is managed through **1Password** as the source of truth:

1. **First provision**: `just provision` checks 1Password for `loaa.barbuto.family` entry
2. **If not found**: Generates secure 32-character password and creates 1Password entry
3. **If found**: Uses existing password from 1Password
4. **Terraform**: Receives password and sets `LOAA_ADMIN_PASSWORD` on droplet

### Accessing Admin Password

To login to the production site:

```bash
# Get the password from 1Password
op read "op://Private/loaa.barbuto.family/password"

# Or use 1Password app to view the entry
```

### Password Rotation

To rotate the admin password:

1. Update the password in 1Password entry `loaa.barbuto.family`
2. Re-run `just provision` (will pick up new password)
3. Or manually update `/opt/loaa/.env` on the droplet and restart

## Database Users (Future)

The User model and UserRepository remain in the codebase for future multi-user support:

```rust
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,  // bcrypt hashed
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### When Regular Users Are Needed

Phase 2+ features may add:
- Kids logging in themselves
- Multiple parent accounts
- Different permission levels

The infrastructure is ready - just create users in the database.

## Security Considerations

### Admin Password Strength

- Generated with `openssl rand -base64 32`
- 32 characters long
- Alphanumeric characters
- No special characters (for easier typing)

### Environment Variable Security

- Password stored in `/opt/loaa/.env` on droplet
- File permissions: `644` (readable by loaa user only)
- Not in source control
- Not in Docker image
- Passed via Terraform (marked as sensitive)

### Session Security

- Uses tower-sessions with secure cookies
- Session data stored server-side
- Cookie contains only session ID
- HTTPS required in production

## Deployment

### Initial Provisioning

```bash
# Just run provision - 1Password handles everything
just provision
```

The script will:
1. ✅ Check for existing `loaa.barbuto.family` in 1Password
2. ✅ Generate password if needed, or use existing
3. ✅ Pass to Terraform as `admin_password` variable
4. ✅ Terraform writes to `/opt/loaa/.env` on droplet

### Manual Password Check

```bash
# SSH into droplet
ssh root@146.190.125.155

# View the password
cat /opt/loaa/.env | grep LOAA_ADMIN_PASSWORD
```

## Login Flow

1. User visits `https://loaa.barbuto.family`
2. Redirected to login page
3. Enters username `admin` and password (from 1Password)
4. Server checks `LOAA_ADMIN_PASSWORD` environment variable
5. On success: Sets `user_id = "admin"` in session
6. Redirected to dashboard

## OAuth Integration

The admin session integrates with OAuth for Claude Desktop:

1. Claude Desktop requests OAuth authorization
2. User must be logged in as admin
3. Admin approves Claude Desktop access
4. JWT token issued for MCP server access

See `OAUTH.md` for details on OAuth flow.
