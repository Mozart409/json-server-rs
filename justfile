# json-server-rs justfile

NAME := "json-server-rs"

# Clear the terminal
clear:
    clear

# Development: format and watch for changes
dev: (fmt) (clear)
    cargo watch -x run

# Start Docker services
up: (clear)
    docker compose up -d --remove-orphans

# Format code using cargo fmt
fmt: (clear)
    cargo fmt

# Lint and fix code using clippy
lint: (clear)
    cargo clippy --fix

# Build release binary
release: (clear)
    cargo build --release

# Run the release binary
run: (clear)
    ./target/release/{{NAME}}

# Start: clear, build release, and run
start: (clear) (release) (run)

# Show help
help:
    @just --list