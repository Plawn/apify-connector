# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

This is a Rust project using **nightly toolchain** (configured in `rust-toolchain.toml`).

```bash
# Build the server binary
cargo build --release --bin server

# Build the cache helper binary
cargo build --release --bin cache

# Run the server locally
cargo run --bin server

# Check/lint
cargo check
cargo clippy
```

## Architecture

This is an **Apify connector service** - an HTTP server that orchestrates Apify actor jobs and transforms their results.

### Binaries

- **server** (`src/main.rs`): Main HTTP server on port 3000 with a single POST `/` endpoint
- **cache** (`src/cache_helper.rs`): Placeholder cache helper utility

### Core Flow

1. Client sends a `JobCreation` request with settings (actor, token, body, mappings) and state
2. Server starts an Apify actor job via the Apify API (`ApiFyClient::start_job`)
3. Server polls for completion (`check_completion`) until the job succeeds or fails
4. Results are downloaded and transformed into `ExportItem` structs using key mappings
5. State is updated using optional Rhai script expressions (prefixed with `$`)
6. Response contains the transformed results and updated state

### Key Modules

- **client.rs**: `ApiFyClient` - HTTP client for Apify API (start jobs, check status, download results)
- **dto.rs**: Data types for Apify API responses, job configuration, and export items
- **mapping_utils.rs**: State update logic with Rhai scripting support for dynamic expressions
- **web_utils.rs**: Axum error handling (`AppError`)

### Data Transformation

- `KeyMapping`: Maps fields from Apify results to `ExportItem` fields (`id`, `content`, `date`, or metadata)
- `StateMapping`: Copies state fields and optionally transforms them using Rhai expressions
- Unmapped fields from source objects are automatically added to metadata

### Dependencies of Note

- **rhai**: Embedded scripting for state transformation expressions
- **axum**: HTTP server framework
- **reqwest**: HTTP client with rustls
