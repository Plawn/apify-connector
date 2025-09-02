use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::dto::{ExportItem, JobCreation};

pub struct Context {
    pub start: DateTime<Utc>,
}

pub fn update_state(
    // unused for now
    _result: &Vec<ExportItem>,
    job: &JobCreation,
    ctx: &Context,
) -> anyhow::Result<String> {
    let mut state: HashMap<String, Value> = serde_json::from_str(&job.state)?;

    if let Some(mapping) = &job.settings.state_mapping {
        for m in mapping {
            match m.update.as_str() {
                "$start_date" => {
                    let k = &m.from;
                    let date_str = ctx.start.format("%Y-%m-%d").to_string();
                    state.insert(k.into(), Value::String(date_str));
                }
                _ => {
                    anyhow::bail!("can't find transformation")
                }
            }
        }
    }

    let e = serde_json::to_string(&state)?;
    Ok(e)
}
