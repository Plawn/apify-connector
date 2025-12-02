# Apify Connector

An HTTP service that orchestrates Apify actor jobs with typed configurations and result transformation.

## Features

- **Typed Actor Configurations**: Each supported Apify actor has a dedicated Rust struct with compile-time validation
- **JSON Schema Discovery**: `GET /actors` endpoint returns all available actors with their JSON Schema definitions
- **Result Transformation**: Maps Apify results to a normalized `ExportItem` format using configurable key mappings
- **State Management**: Supports state persistence between runs with Rhai scripting for dynamic updates

## API Endpoints

### `POST /` - Run a Job

Execute an Apify actor job with typed configuration.

**Request Body:**
```json
{
  "settings": {
    "actor_config": {
      "actor_type": "web_scraper",
      "config": {
        "startUrls": ["https://example.com"],
        "maxPages": 50,
        "useApifyProxy": true
      }
    },
    "token": "your_apify_api_token",
    "key_mapping": [
      { "from": "title", "to": "content", "kind": "String" },
      { "from": "date", "to": "date", "kind": { "Date": { "format": "%Y-%m-%d" } } },
      { "from": "url", "to": "id", "kind": "String" }
    ],
    "state_mapping": [
      { "from": "last_date", "to": "start_date", "update": "$format_date(start_date, \"%Y-%m-%d\")" }
    ]
  },
  "state": "{}"
}
```

**Response:**
```json
{
  "state": "{\"last_date\": \"2024-01-15\"}",
  "result": [
    {
      "id": "https://example.com/page1",
      "content": "Page Title",
      "date": "2024-01-15T00:00:00Z",
      "metadata": {}
    }
  ]
}
```

### `GET /actors` - List Available Actors

Returns all supported actors with their JSON Schema definitions.

**Response:**
```json
[
  {
    "actor_type": "web_scraper",
    "actor_name": "apify/web-scraper",
    "schema": {
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "WebScraperConfig",
      "description": "Configuration for the Apify Web Scraper actor...",
      "type": "object",
      "required": ["startUrls"],
      "properties": {
        "startUrls": {
          "description": "URLs to start scraping from",
          "type": "array",
          "items": { "type": "string" }
        },
        "maxPages": {
          "description": "Maximum pages to crawl",
          "type": "integer",
          "format": "uint32"
        }
      }
    }
  }
]
```

## Supported Actors

### Web Scraper (`web_scraper`)
Actor: `apify/web-scraper`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `startUrls` | `string[]` | Yes | - | URLs to start scraping from |
| `maxPages` | `u32` | No | `100` | Maximum pages to crawl |
| `contentSelector` | `string` | No | - | CSS selector for content extraction |
| `useApifyProxy` | `bool` | No | `false` | Whether to use Apify proxy |

### Google Search (`google_search`)
Actor: `apify/google-search-scraper`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `queries` | `string[]` | Yes | - | Search queries to execute |
| `maxResults` | `u32` | No | `10` | Maximum results per query (1-100) |
| `language` | `string` | No | `"en"` | Language code (e.g., "en", "fr") |
| `countryCode` | `string` | No | - | Country code for localized results |

### Instagram (`instagram`)
Actor: `apify/instagram-scraper`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `usernames` | `string[]` | Yes | - | Instagram usernames to scrape |
| `maxPosts` | `u32` | No | `50` | Maximum posts per profile |
| `includeProfileInfo` | `bool` | No | `false` | Include profile information |
| `includeComments` | `bool` | No | `false` | Include comments on posts |

## Adding a New Actor

1. Create `src/actors/new_actor.rs`:
```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::ActorMetadata;

/// Description of your actor.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewActorConfig {
    /// Field description (becomes JSON Schema description)
    pub required_field: String,
    #[serde(default = "default_value")]
    pub optional_field: u32,
}

fn default_value() -> u32 { 10 }

impl NewActorConfig {
    pub fn metadata() -> ActorMetadata {
        ActorMetadata {
            actor_type: "new_actor",
            actor_name: "apify/new-actor",
            schema: schemars::schema_for!(NewActorConfig),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.required_field.is_empty() {
            return Err("required_field cannot be empty".into());
        }
        Ok(())
    }
}
```

2. Update `src/actors/mod.rs`:
```rust
mod new_actor;
pub use new_actor::NewActorConfig;

// Add to ActorConfig enum
#[serde(rename = "new_actor")]
NewActor(NewActorConfig),

// Add match arms in actor_name(), validate(), to_body()

// Add to list_available_actors()
NewActorConfig::metadata(),
```

## Build & Run

```bash
# Build
cargo build --release

# Run server (port 3000)
cargo run --bin server

# Run tests
cargo test
```

## Docker

```bash
docker build -t apify-connector .
docker run -p 3000:3000 apify-connector
```

## Project Structure

```
src/
├── actors/
│   ├── mod.rs           # ActorConfig enum, ActorMetadata, list_available_actors()
│   ├── web_scraper.rs   # WebScraperConfig
│   ├── google_search.rs # GoogleSearchConfig
│   └── instagram.rs     # InstagramScraperConfig
├── client.rs            # ApiFyClient - HTTP client for Apify API
├── dto.rs               # Data types (Settings, JobCreation, ExportItem, etc.)
├── mapping_utils.rs     # State update logic with Rhai scripting
├── web_utils.rs         # Axum error handling
├── lib.rs               # Library exports
└── main.rs              # HTTP server and handlers
```

## Configuration

### Key Mapping

Maps fields from Apify results to `ExportItem`:
- `id` - Unique identifier
- `content` - Main content field
- `date` - Date field (requires `Date` kind with format)
- Any other `to` value goes into `metadata`

### State Mapping

Updates state between runs. The `update` field supports Rhai expressions prefixed with `$`:
- `$format_date(start_date, "%Y-%m-%d")` - Format a date
- `$sub_days(start_date, 7)` - Subtract days from a date

## Requirements

- Rust nightly (see `rust-toolchain.toml`)
- Apify API token
