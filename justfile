# Default recipe - show available commands
default:
    @just --list

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run a specific test by name
test-one NAME:
    cargo test {{NAME}} -- --nocapture

# Check compilation without building
check:
    cargo check

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the server
run:
    cargo run --bin server

# Run the server in release mode
run-release:
    cargo run --release --bin server

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Clean build artifacts
clean:
    cargo clean

# Run all checks (fmt, lint, test)
ci: fmt-check lint test

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run the server
watch-run:
    cargo watch -x 'run --bin server'
