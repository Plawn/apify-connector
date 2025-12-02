use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::DatasetId;

#[derive(Debug, Deserialize)]
pub struct RunId(pub String);

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(rename = "actId")]
    pub act_id: String,
    #[serde(rename = "buildId")]
    pub build_id: String,
    #[serde(rename = "buildNumber")]
    pub build_number: String,
    #[serde(rename = "containerUrl")]
    pub container_url: String,
    #[serde(rename = "defaultDatasetId")]
    pub default_dataset_id: DatasetId,
    #[serde(rename = "defaultKeyValueStoreId")]
    pub default_key_value_store_id: String,
    #[serde(rename = "defaultRequestQueueId")]
    pub default_request_queue_id: String,
    #[serde(rename = "finishedAt")]
    pub finished_at: Option<String>,
    #[serde(rename = "generalAccess")]
    pub general_access: String,
    #[serde(rename = "id")]
    pub id: RunId,
    #[serde(rename = "meta")]
    pub meta: Meta,
    #[serde(rename = "options")]
    pub options: Options,
    #[serde(rename = "platformUsageBillingModel")]
    pub platform_usage_billing_model: String,
    #[serde(rename = "pricingInfo")]
    pub pricing_info: PricingInfo,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "stats")]
    pub stats: Stats,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    #[serde(rename = "origin")]
    pub origin: String,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Options {
    #[serde(rename = "build")]
    build: String,
    #[serde(rename = "diskMbytes")]
    pub disk_m_bytes: u32,
    #[serde(rename = "maxItems")]
    pub max_items: u32,
    #[serde(rename = "memoryMbytes")]
    pub memory_m_bytes: u32,
    #[serde(rename = "timeoutSecs")]
    pub timeout_secs: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PricingInfo {
  #[serde(rename = "apifyMarginPercentage")]
    pub apify_margin_percentage: f64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "pricePerUnitUsd")]
    pub price_per_unit_usd: f64,
    #[serde(rename = "pricingModel")]
    pub pricing_model: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "unitName")]
    pub unit_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
  #[serde(rename = "computeUnits")]
    pub compute_units: u32,
    #[serde(rename = "inputBodyLen")]
    pub input_body_len: u32,
    #[serde(rename = "migrationCount")]
    pub migration_count: u32,
    #[serde(rename = "rebootCount")]
    pub reboot_count: u32,
    #[serde(rename = "restartCount")]
    pub restart_count: u32,
    #[serde(rename = "resurrectCount")]
    pub resurrect_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct Root {
    pub data: Data,
}
   

#[derive(Serialize, Clone, Debug)]
pub struct ExportItem {
    pub id: Option<String>,
    pub content: String,
    #[serde(with = "jackson")]
    pub date: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub state: String,
    pub result: Vec<ExportItem>,
}

mod jackson {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     // let s = String::deserialize(deserializer)?;
    //     let s: String =Œ Option::deserialize(deserializer)?;
    //     if let Some(s) = s {
    //         return Ok(Some(
    //             Utc.datetime_from_str(&s, FORMAT)
    //         .map_err(serde::de::Error::custom)?
    //         ))
    //     }

    //     Ok(None)
    // }
}

#[derive(Debug, Deserialize)]
pub enum DataKind {
    Date { format: String },
    String,
}

#[derive(Debug, Deserialize)]
pub struct KeyMapping {
    pub from: String,
    pub to: String,
    pub kind: DataKind,
}

#[derive(Debug, Deserialize)]
pub struct StateMapping {
    /// copy from this field
    pub from: String,
    /// to this field
    pub to: String,
    /// at the end map to this value
    pub update: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    /// Actor configuration (parsed based on path parameter actor_type)
    pub actor_config: Value,
    pub token: String,
    pub key_mapping: Vec<KeyMapping>,
    pub state_mapping: Option<Vec<StateMapping>>,
}

/// job with all settings and state
#[derive(Debug, Deserialize)]
pub struct JobCreation {
    pub settings: Settings,
    /// Json encoded state
    pub state: String,
}

/// Settings for running an arbitrary Apify actor
#[derive(Debug, Deserialize)]
pub struct ArbitraryActorSettings {
    /// The Apify actor ID (e.g., "apify/web-scraper" or "Hvp4YfFGyLM635Q2F")
    pub actor_id: String,
    /// Actor input configuration as raw JSON
    pub actor_input: Value,
    pub token: String,
    pub key_mapping: Vec<KeyMapping>,
    pub state_mapping: Option<Vec<StateMapping>>,
}

/// Job request for running an arbitrary Apify actor
#[derive(Debug, Deserialize)]
pub struct ArbitraryActorJob {
    pub settings: ArbitraryActorSettings,
    /// Json encoded state
    pub state: String,
}
