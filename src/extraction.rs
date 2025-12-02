use std::collections::{HashMap, HashSet};

use anyhow::Context;
use chrono::{NaiveDate, TimeZone, Utc};
use serde_json::Value;

use crate::dto::{DataKind, ExportItem, KeyMapping};

/// Extracts a Vec<ExportItem> from JSON array data using key mappings.
pub fn extract_export_items(
    data: Vec<Value>,
    key_mappings: &[KeyMapping],
) -> anyhow::Result<Vec<ExportItem>> {
    let items = data
        .iter()
        .filter_map(|item_value| extract_single_export_item(item_value, key_mappings).ok())
        .collect();
    Ok(items)
}

/// Extracts a single ExportItem from a JSON object using key mappings.
fn extract_single_export_item(
    data: &Value,
    key_mappings: &[KeyMapping],
) -> anyhow::Result<ExportItem> {
    let map = data
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Item in array is not a JSON object"))?;

    let mut id = None;
    let mut content = None;
    let mut date = None;
    let mut metadata = HashMap::new();
    let mut mapped_keys = HashSet::new();

    for mapping in key_mappings {
        if let Some(value) = map.get(&mapping.from) {
            mapped_keys.insert(&mapping.from);

            match mapping.to.as_str() {
                "id" => id = value.as_str().map(String::from),
                "content" => content = value.as_str().map(String::from),
                "date" => {
                    if let DataKind::Date { format } = &mapping.kind
                        && let Some(s) = value.as_str()
                    {
                        let parsed_date =
                            NaiveDate::parse_from_str(s, format).with_context(|| {
                                format!("Failed to parse date '{}' with format '{}'", s, format)
                            })?;
                        date =
                            Some(Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap()));
                    }
                }
                _ => {
                    if let Some(s) = value.as_str() {
                        metadata.insert(mapping.to.clone(), s.to_string());
                    }
                }
            }
        }
    }

    // Collect unmapped fields into metadata
    for (key, value) in map {
        if !mapped_keys.contains(key)
            && let Some(s) = value.as_str()
        {
            metadata.insert(key.clone(), s.to_string());
        }
    }

    Ok(ExportItem {
        id,
        content: content.ok_or_else(|| anyhow::anyhow!("Missing 'content' field in an item"))?,
        date: date.ok_or_else(|| anyhow::anyhow!("Missing or invalid 'date' field in an item"))?,
        metadata,
    })
}
