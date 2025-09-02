use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rhai::{Engine, Scope};
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
    // state should be parsed once
    let mut state: HashMap<String, Value> = serde_json::from_str(&job.state)?;
    // engine should be created once
    let mut engine = Engine::new();
    let mut scope = Scope::new();
    scope.push("start_date", ctx.start);
    engine.register_fn("format_date", |d: DateTime<Utc>, format: &str| {
        d.format(format).to_string()
    });
    engine.register_fn("sub_days", |d: DateTime<Utc>, days: i64| {
        d - Duration::days(days)
    });
    if let Some(mapping) = &job.settings.state_mapping {
        for m in mapping {
            let result = if m.update.starts_with("$") {
                let s = &m.update[1..];
                let result: String = engine
                    .eval_with_scope(&mut scope, s)
                    .unwrap();
                result
            } else {
                m.from.to_string()
            };
            state.insert(m.from.to_string(), Value::String(result));
        }
    }

    let e = serde_json::to_string(&state)?;
    Ok(e)
}
