use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ActorMetadata;

/// Configuration for the TripAdvisor Reviews Scraper actor.
/// Scrapes reviews from TripAdvisor attraction, restaurant, or hotel pages.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TripAdvisorConfig {
    /// TripAdvisor URL to scrape reviews from
    pub url: String,
    /// Filter reviews by rating (e.g., ["ALL_REVIEW_RATINGS"] or ["5", "4"])
    #[serde(default = "default_review_ratings")]
    pub review_ratings: Vec<String>,
    /// Filter reviews by language (e.g., ["ALL_REVIEW_LANGUAGES"] or ["en", "fr"])
    #[serde(default = "default_review_languages")]
    pub reviews_languages: Vec<String>,
    /// Maximum number of reviews to scrape (0 = unlimited)
    #[serde(default)]
    pub max_reviews: u32,
    /// Include reviewer details
    #[serde(default)]
    pub include_reviewer_info: bool,
}

fn default_review_ratings() -> Vec<String> {
    vec!["ALL_REVIEW_RATINGS".to_string()]
}

fn default_review_languages() -> Vec<String> {
    vec!["ALL_REVIEW_LANGUAGES".to_string()]
}

impl TripAdvisorConfig {
    pub fn metadata() -> ActorMetadata {
        ActorMetadata {
            actor_type: "tripadvisor",
            actor_name: "Hvp4YfFGyLM635Q2F",
            schema: schemars::schema_for!(TripAdvisorConfig),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("url cannot be empty".into());
        }
        if !self.url.contains("tripadvisor") {
            return Err(format!(
                "URL does not appear to be a TripAdvisor URL: {}",
                self.url
            ));
        }
        Ok(())
    }

    /// Converts to Apify API format (transforms single url to startUrls array)
    pub fn to_apify_body(&self) -> Result<HashMap<String, Value>, serde_json::Error> {
        let mut body = HashMap::new();

        // Transform single url to startUrls array format
        let start_url = serde_json::json!({
            "url": self.url,
            "method": "GET"
        });
        body.insert("startUrls".to_string(), Value::Array(vec![start_url]));

        body.insert(
            "reviewRatings".to_string(),
            serde_json::to_value(&self.review_ratings)?,
        );
        body.insert(
            "reviewsLanguages".to_string(),
            serde_json::to_value(&self.reviews_languages)?,
        );

        if self.max_reviews > 0 {
            body.insert(
                "maxReviews".to_string(),
                Value::Number(self.max_reviews.into()),
            );
        }

        if self.include_reviewer_info {
            body.insert("includeReviewerInfo".to_string(), Value::Bool(true));
        }

        Ok(body)
    }
}
