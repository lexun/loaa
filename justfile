# List all available recipes
default:
    @just --list --unsorted

# Development workflow: clean, seed, and start web server
[group('workflow')]
dev: clean seed web

# Run the web server
[group('server')]
web:
    cargo run --bin simple_server

# Run all tests
[group('testing')]
test:
    cargo test

# Seed the database with initial data
[group('database')]
seed:
    cargo run --bin seed

# Clean the database (WARNING: deletes all data!)
[group('database')]
clean:
    rm -rf .data/web-db
