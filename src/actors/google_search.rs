use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ActorMetadata;

/// Configuration for the Google Search Scraper actor.
/// Scrapes Google search results for given queries.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearchConfig {
    /// Search queries to execute
    pub queries: Vec<String>,
    /// Maximum results per query (1-100)
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    /// Language code (e.g., "en", "fr")
    #[serde(default = "default_language")]
    pub language: String,
    /// Country code for localized results (e.g., "us", "uk")
    pub country_code: Option<String>,
}

fn default_max_results() -> u32 {
    10
}

fn default_language() -> String {
    "en".to_string()
}

impl GoogleSearchConfig {
    pub fn metadata() -> ActorMetadata {
        ActorMetadata {
            actor_type: "google_search",
            actor_name: "apify/google-search-scraper",
            schema: schemars::schema_for!(GoogleSearchConfig),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.queries.is_empty() {
            return Err("queries cannot be empty".into());
        }
        for query in &self.queries {
            if query.trim().is_empty() {
                return Err("query cannot be empty".into());
            }
        }
        if self.max_results == 0 || self.max_results > 100 {
            return Err("max_results must be between 1 and 100".into());
        }
        if let Some(ref code) = self.country_code
            && code.len() != 2
        {
            return Err("country_code must be 2 characters".into());
        }
        Ok(())
    }
}
