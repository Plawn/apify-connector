use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ActorMetadata;

/// Configuration for the Instagram Scraper actor.
/// Scrapes Instagram profiles and posts.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InstagramScraperConfig {
    /// Instagram usernames to scrape
    pub usernames: Vec<String>,
    /// Maximum posts per profile
    #[serde(default = "default_max_posts")]
    pub max_posts: u32,
    /// Include profile information
    #[serde(default)]
    pub include_profile_info: bool,
    /// Include comments on posts
    #[serde(default)]
    pub include_comments: bool,
}

fn default_max_posts() -> u32 {
    50
}

impl InstagramScraperConfig {
    pub fn metadata() -> ActorMetadata {
        ActorMetadata {
            actor_type: "instagram",
            actor_name: "apify/instagram-scraper",
            schema: schemars::schema_for!(InstagramScraperConfig),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.usernames.is_empty() {
            return Err("usernames cannot be empty".into());
        }
        for username in &self.usernames {
            if username.trim().is_empty() {
                return Err("username cannot be empty".into());
            }
            if username.contains(' ') || username.len() > 30 {
                return Err(format!("Invalid Instagram username: {}", username));
            }
        }
        if self.max_posts == 0 {
            return Err("max_posts must be greater than 0".into());
        }
        Ok(())
    }
}
