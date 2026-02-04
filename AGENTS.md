# AGENTS.md - Developer Guide for json-server-rs

## Project Overview

This is a simple JSON server written in Rust using the axum framework. It serves JSON files from a specified directory via HTTP endpoints.

## Build and Development Commands

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build and run
cargo run

# Build with watch mode
cargo watch -x build
```

### Testing
```bash
# Run all tests
cargo test

# Run a single test by name
cargo test test_name_here

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test --test test_file_name
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Run clippy and automatically fix issues
cargo clippy --fix

# Run clippy with pedantic warnings (as configured)
cargo clippy -- -W clippy::pedantic
```

### Running the Server
```bash
# Run in debug mode
cargo run

# Run with custom port and data directory
cargo run -- --port 3000 --data-dir ./data

# Run release build
./target/release/json-server-rs

# Run with docker compose
docker compose up -d --remove-orphans
```

## Code Style Guidelines

### Project Structure
- **Main entry point**: `src/main.rs`
- **Data directory**: `./data` (contains JSON files to serve)
- **Configuration**: Command-line arguments via `clap`
- **No external tests** - all logic is currently in `main.rs`

### Import Style
- Group imports logically: std → external crates → internal modules
- Use nested imports with braces for related items from the same crate
- Place each group of imports in alphabetical order where possible

**Preferred:**
```rust
use std::fs::{self};
use std::net::SocketAddr;
use std::path::Path as fsPath;
use std::sync::Arc;

use axum::{
    extract::Path, http::StatusCode, response::Html, response::IntoResponse, routing::get, Json,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
```

### Linting Configuration
The project uses strict linting rules:
- `#![forbid(unsafe_code)]` - Unsafe code is prohibited
- `#![warn(clippy::pedantic, clippy::cargo)]` - Enable pedantic and cargo lints
- `#![allow(clippy::unused_async, clippy::unnecessary_wraps)]` - Explicitly allowed lints

**Always keep these linting directives at the top of `main.rs`**

### Naming Conventions
- **Variables and functions**: `snake_case`
- **Types and structs**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Acronyms**: Treat as single word (e.g., `ApiServer` not `APIServer`)

### Type Annotations
- Use explicit types for struct fields
- Use type inference for local variables where the type is obvious
- Use `impl IntoResponse` for handler return types when appropriate
- Use `async fn` for all HTTP handlers

### Error Handling
The project uses `color-eyre` for error handling with rich error messages and context:
- Use `eyre::Result<T>` as return type for fallible operations
- Use `.context()` to add context to errors
- Initialize color-eyre in `main()` with `color_eyre::install()?`
- Avoid `unwrap()` and `expect()` in favor of `?` operator with context

**Required imports:**
```rust
use color_eyre::eyre::{self, Context};
```

**Preferred patterns:**
```rust
// For functions that can fail
fn fallible_operation() -> eyre::Result<T> {
    // ...
}

// Adding context to errors
let data = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read file: {}", path))?;

// For directory operations
let entries = fs::read_dir(&data_dir)
    .with_context(|| format!("Failed to read directory: {}", data_dir))?;

// For parsing and validation
let entry = entry.context("Failed to read directory entry")?;
let file_name = path.file_stem()
    .context("Failed to get file stem")?
    .to_str()
    .context("Invalid filename encoding")?;
```

**Error handling rules:**
- Always provide context with `.context()` or `.with_context()`
- Use meaningful error messages that help identify what failed and why
- Convert Result types using `?` operator instead of unwrapping
- For HTTP handlers, convert errors to appropriate HTTP status codes
- Use `Ok(())` to return success from functions returning `eyre::Result<()>`

### Async/Await
- All HTTP handlers must be async
- Use `tokio::main` for the entry point
- Use `Arc<T>` for shared state in async contexts
- Prefer async/await over callback-style code

### Formatting
- **Line width**: Default (100 characters)
- **Indentation**: 4 spaces
- **Line endings**: LF (Unix-style)
- **Trailing commas**: Always use trailing commas in multi-line structs, enums, and match arms

### Struct Definitions
- Place each field on its own line for multi-field structs
- Use `derive` macros for common traits (`Debug`, `Clone`, `Serialize`, `Deserialize`)
- Document public structs with rustdoc comments

### Function Documentation
- Document all public functions with rustdoc (`///`)
- Include examples in documentation for complex functions
- Use imperative mood for function descriptions ("Builds" not "Build")

### Clap Args
- Use `#[arg(short, long, default_value_t = ...)]` for CLI arguments
- Provide `#[command(author, version, about, long_about = None)]` for the main struct
- Use descriptive help text for each argument

### Tracing and Logging
- Use `tracing` macros: `tracing::debug!`, `tracing::warn!`, `tracing::error!`
- Include context in log messages
- Use structured logging with key-value pairs when appropriate

**Preferred:**
```rust
tracing::debug!(data_dir = %data_dir, "Loading data from directory");
```

### Dependencies
- Keep dependencies minimal
- Check for unused dependencies with `cargo udeps`
- Use feature flags to minimize compilation time
- Pin major versions in `Cargo.toml` for stability

## Git Workflow

### Commit Messages
- Use present tense: "Add feature" not "Added feature"
- Keep subject line under 50 characters
- Include detailed description for complex changes
- Reference issues with "Fixes #123" or "Relates to #456"

### Before Committing
```bash
# Format code
cargo fmt

# Run linter
cargo clippy --fix

# Run tests
cargo test

# Check for build errors
cargo check
```

## Common Tasks

### Adding a New Endpoint
1. Add route to the `Router` in `main()`
2. Create async handler function
3. Return type should implement `IntoResponse`
4. Use `State` for accessing shared state
5. Add appropriate HTTP status codes

### Adding a New CLI Argument
1. Add field to `Args` struct with `#[arg(...)]` attributes
2. Use the new argument in `main()`
3. Update documentation if needed

### Serving New JSON Data
1. Place JSON file in the `./data` directory
2. File is automatically discovered on startup
3. Access via `/api/{filename}` endpoint (without .json extension)

## Development Tools

This project includes a flake.nix for reproducible development environment with:
- Rust (latest stable with rust-analyzer)
- bacon (background code checker)
- lefthook (Git hooks manager)
- just (command runner)

Use `nix develop` to enter the development shell.
