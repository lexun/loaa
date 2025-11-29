# List all available recipes
default:
    @just --list --unsorted

# Start all services in foreground with TUI (recommended for development)
[group('services')]
start:
    devenv up

# Start all services in background (detached mode)
[group('services')]
start-bg:
    devenv processes up --detach

# Attach to running services TUI
[group('services')]
attach:
    process-compose attach

# Stop all services gracefully
[group('services')]
stop:
    #!/usr/bin/env bash
    # First try graceful shutdown
    process-compose down 2>/dev/null || true
    sleep 1
    # Kill process-compose if still running (devenv adds --keep-project flag)
    pkill -f "process-compose.*devenv" || true
    # Clean up any orphaned service processes
    pkill -f "cargo-leptos.*watch" || true
    pkill -f "target/debug/loaa-web" || true
    pkill -f "surreal start.*127.0.0.1:8000" || true

# Restart all services (stops and starts in background)
[group('services')]
restart:
    #!/usr/bin/env bash
    echo "Restarting services..."
    # Stop everything
    process-compose down 2>/dev/null || true
    sleep 1
    pkill -f "process-compose.*devenv" || true
    pkill -f "cargo-leptos.*watch" || true
    pkill -f "target/debug/loaa-web" || true
    pkill -f "surreal start.*127.0.0.1:8000" || true
    # Start fresh in background
    devenv processes up --detach
    echo "Services restarted in background. Run 'just attach' to view TUI."

# View service logs (specify service: db, web, or leave empty for combined)
[group('services')]
logs service='':
    #!/usr/bin/env bash
    if [ -z "{{service}}" ]; then
        # Show combined logs
        if [ -f .devenv/state/process-compose/process-compose.log ]; then
            tail -f .devenv/state/process-compose/process-compose.log
        else
            echo "No log files found. Services may not be running yet."
            echo "Run 'just start' to start services."
        fi
    else
        # Show specific service logs
        LOG_FILE=".devenv/state/process-compose/{{service}}.log"
        if [ -f "$LOG_FILE" ]; then
            tail -f "$LOG_FILE"
        else
            echo "Log file not found: $LOG_FILE"
            echo "Available services: $(ls .devenv/state/process-compose/*.log 2>/dev/null | xargs -n1 basename | sed 's/.log$//' | tr '\n' ' ' || echo 'none')"
        fi
    fi

# Run all tests
[group('testing')]
test:
    cargo test

# Reset database (clean + seed)
[group('database')]
reset:
    #!/usr/bin/env bash
    export LOAA_DB_MODE="${LOAA_DB_MODE:-embedded}"
    export LOAA_DB_PATH="${LOAA_DB_PATH:-./data/loaa.db}"
    echo "Cleaning database at $LOAA_DB_PATH..."
    rm -rf "$LOAA_DB_PATH"
    echo "Seeding database with test data..."
    cargo run -p loaa-web --bin seed --features ssr -- --with-transactions

# Seed the database with initial data
[group('database')]
seed:
    #!/usr/bin/env bash
    export LOAA_DB_MODE="${LOAA_DB_MODE:-embedded}"
    export LOAA_DB_PATH="${LOAA_DB_PATH:-./data/loaa.db}"
    echo "Seeding database..."
    cargo run -p loaa-web --bin seed --features ssr

# Clean the database (WARNING: deletes all data!)
[group('database')]
clean:
    #!/usr/bin/env bash
    export LOAA_DB_PATH="${LOAA_DB_PATH:-./data/loaa.db}"
    echo "Cleaning database at $LOAA_DB_PATH..."
    rm -rf "$LOAA_DB_PATH"
    echo "Database cleaned. Run 'just seed' to populate with fresh data."

# Trigger GitHub Actions deployment workflow
[group('deployment')]
deploy:
    #!/usr/bin/env bash
    if ! command -v gh &> /dev/null; then
        echo "❌ GitHub CLI (gh) not found. Install it with:"
        echo "   brew install gh"
        exit 1
    fi
    echo "🚀 Triggering deployment via GitHub Actions..."
    gh workflow run deploy.yml
    echo "✅ Deployment triggered!"
    echo "   View progress: gh run watch"
    echo "   Or visit: https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/actions"

# Setup infrastructure (one-time, via GitHub Actions)
[group('deployment')]
setup-infra domain do_token ssh_fingerprint:
    #!/usr/bin/env bash
    if ! command -v gh &> /dev/null; then
        echo "❌ GitHub CLI (gh) not found. Install it with:"
        echo "   brew install gh"
        exit 1
    fi
    echo "🏗️  Setting up infrastructure via GitHub Actions..."
    gh workflow run setup-infrastructure.yml \
        -f domain="{{domain}}" \
        -f do_token="{{do_token}}" \
        -f ssh_key_fingerprint="{{ssh_fingerprint}}"
    echo "✅ Infrastructure setup triggered!"
    echo "   View progress: gh run watch"

# Provision infrastructure locally with OpenTofu
[group('deployment')]
provision:
    #!/usr/bin/env bash
    cd terraform

    # Check for 1Password CLI
    if ! command -v op &> /dev/null; then
        echo "❌ 1Password CLI (op) not found. Install it with:"
        echo "   brew install --cask 1password-cli"
        exit 1
    fi

    echo "🔐 Fetching credentials from 1Password..."

    # Set account for personal 1Password (using USER ID from account list)
    export OP_ACCOUNT="GHEB2AKKDRELLL25PDHYRAXNFY"

    # Get DO_TOKEN from 1Password
    DO_TOKEN=$(op read "op://Private/digitalocean.com/terraform" 2>&1)
    if [ $? -ne 0 ]; then
        echo "❌ Failed to fetch DO_TOKEN from 1Password"
        echo "   Error: $DO_TOKEN"
        echo ""
        echo "   Please sign in first:"
        echo "   eval \$(op signin --account GHEB2AKKDRELLL25PDHYRAXNFY)"
        exit 1
    fi

    # Get SSH fingerprint from 1Password (using field ID because @ is invalid in references)
    SSH_KEY_FINGERPRINT=$(op read "op://Private/digitalocean.com/wsm5l6knqha33ocwvfbubzvmku" 2>&1)
    if [ $? -ne 0 ]; then
        echo "❌ Failed to fetch SSH fingerprint from 1Password"
        echo "   Error: $SSH_KEY_FINGERPRINT"
        exit 1
    fi

    # Hardcode domain
    DOMAIN="loaa.barbuto.family"

    # Check if admin password exists in 1Password, create if not
    echo "🔑 Checking for admin password in 1Password..."
    ADMIN_PASSWORD=$(op read "op://Private/$DOMAIN/password" 2>/dev/null)

    if [ -z "$ADMIN_PASSWORD" ]; then
        echo "📝 Admin password not found. Creating new entry in 1Password..."

        # Generate secure password
        ADMIN_PASSWORD=$(openssl rand -base64 32 | tr -d '=+/' | cut -c1-32)

        # Create 1Password item for the domain
        op item create \
            --category=login \
            --title="$DOMAIN" \
            --vault=Private \
            --url="https://$DOMAIN" \
            "username=admin" \
            "password=$ADMIN_PASSWORD" \
            > /dev/null 2>&1

        if [ $? -eq 0 ]; then
            echo "✅ Created 1Password entry: $DOMAIN"
            echo "   Username: admin"
            echo "   Password: (stored in 1Password)"
        else
            echo "❌ Failed to create 1Password entry"
            exit 1
        fi
    else
        echo "✅ Admin password found in 1Password"
    fi

    echo "✅ Credentials loaded"
    echo "   Domain: $DOMAIN"
    echo "   SSH Key: ${SSH_KEY_FINGERPRINT:0:20}..."

    # Create tfvars file
    cat > terraform.tfvars << EOF
    do_token            = "$DO_TOKEN"
    domain              = "$DOMAIN"
    ssh_key_fingerprint = "$SSH_KEY_FINGERPRINT"
    admin_password      = "$ADMIN_PASSWORD"
    EOF

    echo ""
    echo "🏗️  Initializing OpenTofu..."
    tofu init

    echo ""
    echo "📋 Planning infrastructure..."
    tofu plan -var-file=terraform.tfvars

    echo ""
    read -p "Apply this plan? (yes/no): " confirm
    if [ "$confirm" = "yes" ]; then
        echo "🚀 Provisioning infrastructure..."
        tofu apply -var-file=terraform.tfvars -auto-approve
        echo ""
        echo "✅ Infrastructure provisioned!"
        echo ""
        tofu output infrastructure_summary
        echo ""
        tofu output dns_instructions
    else
        echo "❌ Cancelled"
    fi

# Update admin password on existing droplet
[group('deployment')]
update-admin-password:
    #!/usr/bin/env bash
    # Check for 1Password CLI
    if ! command -v op &> /dev/null; then
        echo "❌ 1Password CLI (op) not found."
        exit 1
    fi

    # Set account
    export OP_ACCOUNT="GHEB2AKKDRELLL25PDHYRAXNFY"

    # Get admin password from 1Password
    echo "🔑 Fetching admin password from 1Password..."
    ADMIN_PASSWORD=$(op read "op://Private/loaa.barbuto.family/password" 2>&1)
    if [ $? -ne 0 ]; then
        echo "❌ Failed to fetch admin password"
        echo "   Error: $ADMIN_PASSWORD"
        exit 1
    fi

    # Get droplet IP from terraform state
    cd terraform
    DROPLET_IP=$(tofu output -raw droplet_ip 2>/dev/null)
    if [ -z "$DROPLET_IP" ]; then
        echo "❌ Could not get droplet IP from terraform state"
        exit 1
    fi

    echo "🌐 Droplet IP: $DROPLET_IP"
    echo "🔐 Updating admin password on droplet..."

    # SSH into droplet and update .env file
    ssh -o StrictHostKeyChecking=accept-new -i ~/.ssh/google_compute_engine root@$DROPLET_IP "
        # Check if password already exists in .env
        if grep -q 'LOAA_ADMIN_PASSWORD' /opt/loaa/.env; then
            # Update existing line
            sed -i 's/^LOAA_ADMIN_PASSWORD=.*/LOAA_ADMIN_PASSWORD=$ADMIN_PASSWORD/' /opt/loaa/.env
            echo '✅ Updated existing LOAA_ADMIN_PASSWORD'
        else
            # Add new line
            echo 'LOAA_ADMIN_PASSWORD=$ADMIN_PASSWORD' >> /opt/loaa/.env
            echo '✅ Added LOAA_ADMIN_PASSWORD'
        fi

        # Restart the application to pick up new password
        cd /opt/loaa
        if [ -f docker-compose.yml ]; then
            docker-compose restart
            echo '✅ Application restarted'
        else
            echo '⚠️  docker-compose.yml not found, skipping restart'
        fi
    "

    echo ""
    echo "✅ Admin password updated successfully!"
    echo "   Login with username: admin"
    echo "   Password from: op read \"op://Private/loaa.barbuto.family/password\""

# Destroy infrastructure
[group('deployment')]
destroy:
    #!/usr/bin/env bash
    cd terraform

    if [ ! -f terraform.tfvars ]; then
        echo "❌ No terraform.tfvars found. Run 'just provision' first."
        exit 1
    fi

    echo "⚠️  WARNING: This will destroy all infrastructure!"
    read -p "Are you sure? (yes/no): " confirm
    if [ "$confirm" = "yes" ]; then
        echo "💥 Destroying infrastructure..."
        tofu destroy -var-file=terraform.tfvars -auto-approve
        echo "✅ Infrastructure destroyed"
    else
        echo "❌ Cancelled"
    fi
