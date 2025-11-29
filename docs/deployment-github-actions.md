# Deployment Guide - GitHub Actions

Everything is automated through GitHub Actions! No local scripts needed.

## Overview

**Deployment Flow:**
1. Push to `main` → GHA builds Docker image → Pushes to ghcr.io → Deploys to Digital Ocean
2. All building happens in GitHub Actions (no Mac/Linux issues!)
3. Container image is the deployment artifact
4. Simple SSH deployment to pull and restart

## One-Time Setup

### Step 1: Install Tools Locally

```bash
# Only terraform needed for initial setup
brew install terraform
```

### Step 2: Digital Ocean Setup

1. **Create Account**: https://digitalocean.com
2. **Get API Token**:
   - Go to: https://cloud.digitalocean.com/account/api/tokens
   - Generate New Token → Name: "Terraform" → Read + Write
   - Copy the token
3. **Add SSH Key**:
   ```bash
   # Generate if you don't have one
   ssh-keygen -t ed25519 -C "your_email@example.com"

   # Add to Digital Ocean
   # https://cloud.digitalocean.com/account/security
   # Copy your public key:
   cat ~/.ssh/id_ed25519.pub
   ```

### Step 3: Push Code to GitHub

```bash
# Create repo on GitHub first, then:
git remote add origin git@github.com:yourusername/loaa.git
git push -u origin main
```

### Step 4: Run Infrastructure Setup (One-Time)

Go to your GitHub repo:
1. Actions → "Setup Infrastructure" → Run workflow
2. Fill in:
   - **Domain**: `loaa.yourdomain.com`
   - **DO Token**: From step 2
   - **SSH Fingerprint**: From Digital Ocean security page

This workflow will:
- Create Digital Ocean droplet
- Set up firewall
- Install Docker
- Get SSL certificate
- Create environment file

### Step 5: Add GitHub Secrets

After infrastructure is created, add these secrets:
(Settings → Secrets and variables → Actions → New repository secret)

- **DROPLET_IP**: From workflow output
- **DROPLET_SSH_KEY**: Your private SSH key
  ```bash
  cat ~/.ssh/id_ed25519
  # Copy everything including BEGIN/END lines
  ```
- **DOMAIN**: `loaa.yourdomain.com`

### Step 6: Configure DNS

Add this A record in Squarespace:
- **Type**: A
- **Host**: `loaa` (or your subdomain)
- **Points to**: `<DROPLET_IP from step 5>`
- **TTL**: 3600

Wait 5-10 minutes for DNS propagation.

## Deployment

### Automatic Deployment

Every push to `main` triggers automatic deployment:

```bash
git add .
git commit -m "Update feature"
git push origin main
```

GitHub Actions will:
1. ✅ Build Docker image (linux/amd64)
2. ✅ Push to ghcr.io
3. ✅ SSH into droplet
4. ✅ Pull new image
5. ✅ Restart services
6. ✅ Run health checks

### Manual Deployment

Go to: Actions → "Build and Deploy" → Run workflow

### Monitor Deployment

- GitHub repo → Actions tab
- See real-time logs
- Get notified on failures

## What Gets Deployed

**Artifact**: Docker image at `ghcr.io/yourusername/loaa:latest`

**Contains**:
- Rust binary (compiled for linux/amd64)
- All dependencies
- SurrealDB embedded
- Web app + MCP server

**Deployment Location**: `/opt/loaa` on droplet

## Managing Your Deployment

### View Logs

```bash
ssh root@<droplet-ip>
cd /opt/loaa
docker-compose logs -f
```

### Restart Services

```bash
ssh root@<droplet-ip>
cd /opt/loaa
docker-compose restart
```

### Check Status

```bash
ssh root@<droplet-ip>
cd /opt/loaa
docker-compose ps
```

### Update Environment Variables

```bash
ssh root@<droplet-ip>
nano /opt/loaa/.env
# Edit variables
docker-compose restart
```

## Costs

- **Digital Ocean Droplet**: $6/month
- **GitHub Actions**: Free (2000 min/month)
- **GitHub Container Registry**: Free (500MB)
- **SSL Certificate**: Free (Let's Encrypt)
- **Total**: $6/month

## Connect from Claude Desktop

### Production (Deployed)

In Claude Desktop settings, add remote MCP server:

1. Open Claude Desktop
2. Settings → MCP Servers → Add Server
3. Fill in:
   - **Name:** Loa'a
   - **Server URL:** `https://loaa.yourdomain.com:3001/mcp`
   - Leave OAuth fields empty (auto-discovery)
4. Click "Add Server"

Claude will:
1. Auto-discover OAuth configuration
2. Redirect you to login page
3. You log in with `admin` / `admin123`
4. Complete authorization
5. Ready to use!

### Local Development

For local testing (not in Claude Desktop UI, use Claude Code MCP integration):

```bash
# Start local server
just restart

# The MCP server is now available at:
# http://127.0.0.1:3001/mcp
```

**Note**: Claude Desktop requires HTTPS for remote servers, so you cannot connect to `localhost` from the UI. Use the deployed version or Claude Code for local development.

### Try It Out

Once connected, ask Claude:
- "List all kids in the system"
- "Create a new task for taking out the trash worth $2.50"
- "Show me Yasu's ledger"
- "Complete the 'Feed Pets' task for Zevi"

### Default Credentials

- Username: `admin`
- Password: `admin123`

**⚠️ IMPORTANT: Change these immediately after first login!**

## Troubleshooting

### Build fails in GitHub Actions

Check Actions tab for errors. Common issues:
- Rust compilation errors (fix locally, test with `cargo check`)
- Missing dependencies (update Dockerfile)

### Deployment fails

Check GitHub Actions logs for SSH errors:
- Verify `DROPLET_SSH_KEY` secret is correct private key
- Ensure droplet is running

### Can't access site

1. Check DNS: `dig loaa.yourdomain.com` should return droplet IP
2. Check firewall: `ssh root@<ip>` then `ufw status`
3. Check services: `docker-compose ps`

### SSL certificate issues

```bash
ssh root@<droplet-ip>
certbot certificates  # Check certificate status
certbot renew        # Renew if needed
```

## Rollback

If deployment breaks:

```bash
ssh root@<droplet-ip>
cd /opt/loaa

# List available images
docker images | grep loaa

# Use a specific tag
docker-compose down
# Edit docker-compose.yml to use specific image tag
docker-compose up -d
```

Or redeploy a previous GitHub commit:
- Go to Actions → Find successful previous workflow → Re-run jobs

## Cleanup / Destroy

To tear down everything:

```bash
cd terraform
terraform destroy -var-file=terraform.tfvars
```

This removes the droplet but keeps your code and Docker images.
