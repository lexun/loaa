# GitHub Actions Deployment

This workflow automatically builds and deploys Loa'a when you push to the main branch.

## How It Works

1. **Build**: Builds Docker image on GitHub's servers (linux/amd64)
2. **Push**: Pushes to GitHub Container Registry (ghcr.io)
3. **Deploy**: SSHs into your Digital Ocean droplet and pulls the new image

## Setup Instructions

### 1. Enable GitHub Container Registry

The registry is enabled by default. Your images will be at:
```
ghcr.io/lexun/loaa:latest
```

### 2. Add GitHub Secrets

Go to your repo settings → Secrets and variables → Actions

Add these secrets:

**DROPLET_IP**
- The IP address of your Digital Ocean droplet
- Example: `164.90.123.45`

**DROPLET_SSH_KEY**
- Your private SSH key (the one you use to SSH into the droplet)
- Get it with: `cat ~/.ssh/id_ed25519`
- Copy the entire key including `-----BEGIN` and `-----END` lines

**DOMAIN**
- Your domain name
- Example: `loaa.yourdomain.com`

### 3. Make Repository Public or Configure Private Access

For **public repos**: No additional setup needed!

For **private repos**:
- The workflow uses `${{ secrets.GITHUB_TOKEN }}` which is automatically provided
- The droplet needs to authenticate to pull private images
- Add this to your droplet's `/opt/loaa/.env`:
  ```bash
  GITHUB_TOKEN=your-personal-access-token
  ```
  Generate a token at: https://github.com/settings/tokens
  - Scopes needed: `read:packages`

### 4. Initial Deployment

The first time, you still need to set up the droplet infrastructure:

```bash
# Run terraform to create droplet
cd terraform
terraform init
terraform apply -var-file=variables.tfvars

# Get the droplet IP and add it to GitHub secrets
terraform output droplet_ip
```

Then configure the droplet manually once:

```bash
# SSH into droplet
ssh root@<droplet-ip>

# Create directory and .env file
mkdir -p /opt/loaa
cat > /opt/loaa/.env << 'EOF'
LOAA_JWT_SECRET=$(openssl rand -base64 32)
LOAA_BASE_URL=https://loaa.yourdomain.com
LOAA_INCLUDE_MCP=true
LOAA_MCP_PORT=3001
EOF

# Set up SSL (one-time)
apt-get update
apt-get install -y certbot
certbot certonly --standalone -d loaa.yourdomain.com --non-interactive --agree-tos -m admin@yourdomain.com

# Add SSL to docker-compose
cat > /opt/loaa/docker-compose.override.yml << 'EOF'
version: '3.8'
services:
  web:
    volumes:
      - /etc/letsencrypt:/etc/letsencrypt:ro
EOF
```

### 5. Deploy!

Now just push to main:

```bash
git add .
git commit -m "Deploy Loa'a"
git push origin main
```

GitHub Actions will:
1. Build the Docker image
2. Push to ghcr.io
3. SSH into your droplet
4. Pull and restart with the new image

## Monitoring Deployments

View deployment status:
- GitHub repo → Actions tab
- See build logs, deployment status, errors

## Manual Deployment

Trigger a deployment without pushing:
- Go to Actions → Build and Deploy → Run workflow

## Costs

- **GitHub Actions**: 2000 minutes/month free (builds take ~5-10 min)
- **GitHub Container Registry**: Free for public repos, 500MB free for private
- **Total**: $0 for public repos!

## Troubleshooting

### "Permission denied" during deployment

Add your SSH key to GitHub secrets. Make sure it's the private key, not the public key.

### "Failed to pull image"

For private repos, make sure the droplet has a GitHub token:
```bash
ssh root@<droplet-ip>
echo "GITHUB_TOKEN=ghp_your_token_here" >> /opt/loaa/.env
```

### Build fails

Check the Actions tab for error logs. Common issues:
- Rust compilation errors (fix locally first)
- Missing dependencies (update Dockerfile)

## Alternative: Use Terraform + GitHub Actions

You can also fully automate infrastructure creation. See `.github/workflows/terraform.yml.example` for a complete setup that creates the droplet too.
