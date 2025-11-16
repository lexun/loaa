# List all available recipes
default:
    @just --list --unsorted

# Start all services (foreground with TUI, -d for background daemon)
[group('services')]
start mode='':
    #!/usr/bin/env bash
    if [ "{{mode}}" = "-d" ]; then
        echo "Starting services in background..."
        PC_DISABLE_TUI=true nohup devenv up > .devenv/state/services.log 2>&1 &
        echo $! > .devenv/state/services.pid
        echo "Services started. PID: $(cat .devenv/state/services.pid)"
        echo "View logs with: just logs"
        echo "Attach TUI with: just start"
    else
        devenv up
    fi

# Stop all services
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

# Restart all services (foreground by default, -d for background)
[group('services')]
restart mode='':
    @just stop
    @just start {{mode}}

# View service logs (all by default, or specify service: db, web)
[group('services')]
logs service='':
    #!/usr/bin/env bash
    if [ -z "{{service}}" ]; then
        # Show combined logs
        if [ -f .devenv/state/services.log ]; then
            tail -f .devenv/state/services.log
        elif [ -f .devenv/state/process-compose/process-compose.log ]; then
            tail -f .devenv/state/process-compose/process-compose.log
        else
            echo "No log files found. Run 'just start' first."
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
    echo "Cleaning database..."
    rm -rf data/loaa.db
    echo "Seeding database..."
    cargo run --bin seed --features ssr

# Seed the database with initial data
[group('database')]
seed:
    cargo run --bin seed --features ssr

# Clean the database (WARNING: deletes all data!)
[group('database')]
clean:
    rm -rf data/loaa.db
