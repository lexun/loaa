# List all available recipes
default:
    @just --list --unsorted

# Start all services (foreground with TUI by default)
[group('services')]
start:
    devenv up

# Stop all services (run from another terminal, or use Ctrl+C in the TUI)
[group('services')]
stop:
    #!/usr/bin/env bash
    echo "Stopping services gracefully..."
    process-compose down --ordered-shutdown || echo "No process-compose instance running"

# Restart all services
[group('services')]
restart:
    @just stop
    @sleep 2
    @just start

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

# Reset database (clean + seed) - requires services to be running
[group('database')]
reset:
    #!/usr/bin/env bash
    echo "Note: This requires 'just start' to be running in another terminal"
    echo ""
    echo "Seeding database with fresh data..."
    cargo run --bin seed --features ssr
    echo ""
    echo "Creating test transactions..."
    cargo run --bin create_transactions --features ssr

# Seed the database with initial data - requires services to be running
[group('database')]
seed:
    #!/usr/bin/env bash
    echo "Note: This requires 'just start' to be running in another terminal"
    echo ""
    cargo run --bin seed --features ssr

# Clean the database (WARNING: deletes all data!)
# Note: Database runs in-memory mode, so restart services to clean
[group('database')]
clean:
    #!/usr/bin/env bash
    echo "Database runs in-memory mode."
    echo "To clean the database, restart services with 'just restart'"
    echo "Then run 'just seed' to populate with fresh data."
