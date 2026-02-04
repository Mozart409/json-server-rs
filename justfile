# json-server-rs justfile

NAME := "json-server-rs"

# Clear the terminal
clear:
    clear

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

# Build Debian package
deb: (clear)
    cargo deb

# Build RPM package
rpm: (clear)
    cargo generate-rpm

# Start: clear, build release, and run
start: (clear) (release) (run)

# Show help
help:
    @just --list
