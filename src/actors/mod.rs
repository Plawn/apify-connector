mod google_search;
mod instagram;
mod tripadvisor;
mod web_scraper;

pub use google_search::GoogleSearchConfig;
pub use instagram::InstagramScraperConfig;
pub use tripadvisor::TripAdvisorConfig;
pub use web_scraper::WebScraperConfig;

use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Metadata about an available actor/scraper with JSON Schema
#[derive(Debug, Clone, Serialize)]
pub struct ActorMetadata {
    pub actor_type: &'static str,
    pub actor_name: &'static str,
    pub schema: RootSchema,
}

/// Returns metadata for all available actors
pub fn list_available_actors() -> Vec<ActorMetadata> {
    vec![
        WebScraperConfig::metadata(),
        GoogleSearchConfig::metadata(),
        InstagramScraperConfig::metadata(),
        TripAdvisorConfig::metadata(),
    ]
}

/// Returns metadata for a specific actor type
pub fn get_actor_metadata(actor_type: &str) -> Option<ActorMetadata> {
    match actor_type {
        "web_scraper" => Some(WebScraperConfig::metadata()),
        "google_search" => Some(GoogleSearchConfig::metadata()),
        "instagram" => Some(InstagramScraperConfig::metadata()),
        "tripadvisor" => Some(TripAdvisorConfig::metadata()),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ActorConfig {
    WebScraper(WebScraperConfig),
    GoogleSearch(GoogleSearchConfig),
    Instagram(InstagramScraperConfig),
    TripAdvisor(TripAdvisorConfig),
}

impl ActorConfig {
    /// Parse an ActorConfig from actor_type path parameter and config JSON
    pub fn from_type_and_config(actor_type: &str, config: Value) -> Result<Self, String> {
        match actor_type {
            "web_scraper" => serde_json::from_value(config)
                .map(ActorConfig::WebScraper)
                .map_err(|e| format!("Invalid web_scraper config: {}", e)),
            "google_search" => serde_json::from_value(config)
                .map(ActorConfig::GoogleSearch)
                .map_err(|e| format!("Invalid google_search config: {}", e)),
            "instagram" => serde_json::from_value(config)
                .map(ActorConfig::Instagram)
                .map_err(|e| format!("Invalid instagram config: {}", e)),
            "tripadvisor" => serde_json::from_value(config)
                .map(ActorConfig::TripAdvisor)
                .map_err(|e| format!("Invalid tripadvisor config: {}", e)),
            _ => Err(format!("Unknown actor type: {}", actor_type)),
        }
    }

    /// Returns the actor_type string for this config
    pub fn actor_type(&self) -> &'static str {
        match self {
            Self::WebScraper(_) => "web_scraper",
            Self::GoogleSearch(_) => "google_search",
            Self::Instagram(_) => "instagram",
            Self::TripAdvisor(_) => "tripadvisor",
        }
    }

    /// Returns the Apify actor identifier for this configuration
    pub fn actor_name(&self) -> &'static str {
        match self {
            Self::WebScraper(_) => "apify/web-scraper",
            Self::GoogleSearch(_) => "apify/google-search-scraper",
            Self::Instagram(_) => "apify/instagram-scraper",
            Self::TripAdvisor(_) => "Hvp4YfFGyLM635Q2F",
        }
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::WebScraper(c) => c.validate(),
            Self::GoogleSearch(c) => c.validate(),
            Self::Instagram(c) => c.validate(),
            Self::TripAdvisor(c) => c.validate(),
        }
    }

    /// Converts the typed config to a HashMap for the API call
    pub fn to_body(&self) -> Result<HashMap<String, Value>, serde_json::Error> {
        match self {
            Self::TripAdvisor(c) => c.to_apify_body(),
            _ => {
                let value = match self {
                    Self::WebScraper(c) => serde_json::to_value(c)?,
                    Self::GoogleSearch(c) => serde_json::to_value(c)?,
                    Self::Instagram(c) => serde_json::to_value(c)?,
                    Self::TripAdvisor(_) => unreachable!(),
                };
                match value {
                    Value::Object(map) => Ok(map.into_iter().collect()),
                    _ => Ok(HashMap::new()),
                }
            }
        }
    }
}
