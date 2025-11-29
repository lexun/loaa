# GitHub Actions Deployment

This workflow automatically builds and deploys Loa'a when you push to the main branch.

## How It Works

1. **Build**: Compiles Rust code and builds Docker image on GitHub's servers (linux/amd64)
2. **Push**: Pushes image to GitHub Container Registry (ghcr.io)
3. **Deploy**: SSHs into Digital Ocean droplet and pulls the new image

## Prerequisites

### Required Tools (Local)
- [1Password CLI](https://developer.1password.com/docs/cli) - For secrets management
- [OpenTofu](https://opentofu.org/) - For infrastructure provisioning (added via devenv)
- [Just](https://github.com/casey/just) - Task runner (added via devenv)

### Required Accounts
- **Digital Ocean** - For hosting the droplet
- **1Password** - For storing secrets (DO token, SSH keys, admin password)
- **Squarespace** (or other DNS provider) - For domain management

## Initial Setup

### 1. Store Secrets in 1Password

Create a **digitalocean.com** login item in your Private vault with:
- Field `terraform`: Your Digital Ocean API token
- Field (SSH fingerprint): Your SSH key fingerprint from DO

The `just provision` command will automatically:
- Generate a secure admin password
- Create a 1Password entry for `loaa.barbuto.family` (or your domain)
- Store the admin password there

### 2. Provision Infrastructure

Run locally:

```bash
just provision
```

This will:
- Create a Digital Ocean droplet ($6/month)
- Create a persistent volume ($1/month)
- Set up firewall rules
- Configure the environment
- Generate admin password in 1Password

### 3. Configure GitHub Secrets

The secrets are automatically configured by the provision script via `gh` CLI:

- **DROPLET_IP**: Droplet IP address
- **DROPLET_SSH_KEY**: Private SSH key from 1Password
- **DOMAIN**: Your domain name

These are used by the `deploy.yml` workflow.

### 4. Configure DNS

Add an A record in Squarespace (or your DNS provider):
- **Type**: A
- **Host**: `loaa` (or your subdomain)
- **Points to**: [DROPLET_IP from provision output]
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
1. ✅ Build Docker image (Rust nightly, all dependencies)
2. ✅ Push to ghcr.io
3. ✅ SSH into droplet
4. ✅ Pull new image
5. ✅ Restart services with zero-downtime
6. ✅ Run health checks

### Manual Deployment

Trigger without pushing:
- Go to: Actions → "Build and Deploy" → Run workflow

## What Gets Deployed

**Container Image**: `ghcr.io/lexun/loaa:latest`

**Contains**:
- Rust web server (Leptos SSR)
- MCP server (embedded mode)
- SurrealDB (embedded RocksDB)
- All dependencies

**Deployment Location**: `/opt/loaa` on droplet

**Persistent Data**: `/mnt/loaa-data/surrealdb` (10GB volume)

## Admin Access

**Login URL**: `https://loaa.barbuto.family` (or your domain)

**Credentials**:
- Username: `admin`
- Password: Stored in 1Password under `loaa.barbuto.family`

To get the admin password:
```bash
op read "op://Private/loaa.barbuto.family/password"
```

## Monitoring Deployments

**View Status**:
- GitHub repo → Actions tab
- Real-time build logs
- Deployment status
- Error notifications

**Check Droplet**:
```bash
# SSH into droplet
ssh root@[DROPLET_IP]

# View logs
cd /opt/loaa
docker-compose logs -f

# Check status
docker-compose ps

# Restart if needed
docker-compose restart
```

## Infrastructure Details

### Costs
- **Digital Ocean Droplet**: $6/month (Basic, 1GB RAM)
- **Persistent Volume**: $1/month (10GB)
- **GitHub Actions**: Free (2000 min/month)
- **GitHub Container Registry**: Free for public repos
- **Total**: $7/month

### Architecture
- **Web Server**: Leptos SSR on port 3000
- **MCP Server**: Embedded HTTP on port 3001
- **Database**: SurrealDB embedded (RocksDB backend)
- **Reverse Proxy**: Caddy (auto-SSL via Let's Encrypt)
- **Data Persistence**: Digital Ocean Volume at `/mnt/loaa-data`

## Troubleshooting

### Build Fails in GitHub Actions

Check Actions tab for errors. Common issues:
- **Rust compilation errors**: Fix locally with `cargo check` first
- **Missing dependencies**: Check Dockerfile has all required packages
- **Version mismatches**: Ensure wasm-bindgen versions match

### Deployment Fails

Check deployment logs in Actions. Common issues:
- **SSH connection failed**: Verify `DROPLET_SSH_KEY` secret is correct private key
- **Cannot pull image**: For private repos, ensure GITHUB_TOKEN permissions
- **Droplet offline**: Check Digital Ocean console

### Site Not Accessible

1. **Check DNS**: `dig loaa.barbuto.family` should return droplet IP
2. **Check firewall**:
   ```bash
   ssh root@[DROPLET_IP]
   ufw status
   ```
3. **Check services**:
   ```bash
   docker-compose ps
   docker-compose logs
   ```

### Database Issues

Database is persisted on the Digital Ocean volume:
- **Location**: `/mnt/loaa-data/surrealdb`
- **Survives**: Droplet rebuilds and restarts
- **Backup**: Regular snapshots via Digital Ocean

## Updating Infrastructure

To modify infrastructure (droplet size, regions, etc.):

1. Edit `terraform/main.tf`
2. Run `just provision` again
3. Terraform will show planned changes
4. Approve to apply changes

## Password Rotation

To rotate the admin password:

1. Update password in 1Password entry `loaa.barbuto.family`
2. Run:
   ```bash
   ssh root@[DROPLET_IP]
   cd /opt/loaa
   # Update .env file
   sed -i 's/^LOAA_ADMIN_PASSWORD=.*/LOAA_ADMIN_PASSWORD=NEW_PASSWORD/' .env
   docker-compose restart
   ```

Or re-provision (will recreate droplet):
```bash
just provision
```

## Emergency Rollback

If a deployment breaks the site:

```bash
ssh root@[DROPLET_IP]
cd /opt/loaa

# List recent images
docker images | grep loaa

# Use a specific previous tag
docker-compose down
# Edit docker-compose.yml to use specific tag (e.g., main-abc1234)
docker-compose up -d
```

Or re-run a previous successful workflow:
- Go to Actions → Find successful deployment → Re-run jobs

## Cleanup / Destroy

To tear down all infrastructure:

```bash
cd terraform
tofu destroy -var-file=terraform.tfvars
```

This removes:
- Droplet
- Persistent volume (⚠️ destroys all data)
- Firewall rules

Docker images in ghcr.io remain and can be manually deleted from GitHub packages.
