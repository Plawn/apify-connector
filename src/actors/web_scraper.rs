use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ActorMetadata;

/// Configuration for the Apify Web Scraper actor.
/// Scrapes web pages starting from given URLs.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebScraperConfig {
    /// URLs to start scraping from
    pub start_urls: Vec<String>,
    /// Maximum pages to crawl
    #[serde(default = "default_max_pages")]
    pub max_pages: u32,
    /// CSS selector for content extraction
    pub content_selector: Option<String>,
    /// Whether to use Apify proxy
    #[serde(default)]
    pub use_apify_proxy: bool,
}

fn default_max_pages() -> u32 {
    100
}

impl WebScraperConfig {
    pub fn metadata() -> ActorMetadata {
        ActorMetadata {
            actor_type: "web_scraper",
            actor_name: "apify/web-scraper",
            schema: schemars::schema_for!(WebScraperConfig),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.start_urls.is_empty() {
            return Err("start_urls cannot be empty".into());
        }
        for url in &self.start_urls {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(format!("Invalid URL: {}", url));
            }
        }
        if self.max_pages == 0 {
            return Err("max_pages must be greater than 0".into());
        }
        Ok(())
    }
}
