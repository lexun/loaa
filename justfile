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
