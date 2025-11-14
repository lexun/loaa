# List all available recipes
default:
    @just --list --unsorted

# Start all services (database + web server)
[group('services')]
up:
    devenv up

# Start all services in background (headless)
[group('services')]
start:
    #!/usr/bin/env bash
    echo "Starting services in background..."
    nohup devenv up > .devenv/state/services.log 2>&1 &
    echo $! > .devenv/state/services.pid
    echo "Services started. PID: $(cat .devenv/state/services.pid)"
    echo "View logs with: just logs"

# Stop all background services
[group('services')]
stop:
    #!/usr/bin/env bash
    if [ -f .devenv/state/services.pid ]; then
        PID=$(cat .devenv/state/services.pid)
        echo "Stopping services (PID: $PID)..."
        kill $PID 2>/dev/null || echo "Process already stopped"
        rm -f .devenv/state/services.pid
    else
        echo "No services.pid file found. Trying to find and stop processes..."
        pkill -f "devenv up" || echo "No devenv processes found"
        pkill -f surreal || echo "No surreal processes found"
        pkill -f simple_server || echo "No web server processes found"
    fi

# Restart all services
[group('services')]
restart: stop start

# View service logs (combined)
[group('services')]
logs:
    #!/usr/bin/env bash
    if [ -f .devenv/state/services.log ]; then
        tail -f .devenv/state/services.log
    elif [ -f .devenv/state/process-compose/process-compose.log ]; then
        tail -f .devenv/state/process-compose/process-compose.log
    else
        echo "No log files found. Run 'just start' first."
    fi

# View logs for a specific service (db or web)
[group('services')]
log service:
    #!/usr/bin/env bash
    LOG_FILE=".devenv/state/process-compose/{{service}}.log"
    if [ -f "$LOG_FILE" ]; then
        tail -f "$LOG_FILE"
    else
        echo "Log file not found: $LOG_FILE"
        echo "Available: $(ls .devenv/state/process-compose/*.log 2>/dev/null || echo 'none')"
    fi

# Run all tests
[group('testing')]
test:
    cargo test

# Reset database (clean + seed)
[group('database')]
reset:
    #!/usr/bin/env bash
    echo "Cleaning database..."
    rm -rf data/loaa.db
    echo "Seeding database..."
    cargo run --bin seed

# Seed the database with initial data
[group('database')]
seed:
    cargo run --bin seed

# Clean the database (WARNING: deletes all data!)
[group('database')]
clean:
    rm -rf data/loaa.db
