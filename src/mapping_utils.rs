use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rhai::{Engine, Scope};
use serde_json::Value;

use crate::dto::{ExportItem, JobCreation, StateMapping};

#[derive(Clone, Debug, Default)]
pub struct Context {
    pub start: DateTime<Utc>,
}

impl Context {
    pub fn new() -> Self {
        Self { start: Utc::now() }
    }
}

/// Core state update logic that can be used by both typed and arbitrary actor jobs
pub fn update_state_core(
    _result: &Vec<ExportItem>,
    state_str: &str,
    state_mapping: Option<&Vec<StateMapping>>,
    ctx: Context,
) -> anyhow::Result<String> {
    let mut state: HashMap<String, Value> = serde_json::from_str(state_str)?;
    let mut engine = Engine::new();
    let mut scope = Scope::new();
    scope.push("start_date", ctx.start);
    engine.register_fn("format_date", |d: DateTime<Utc>, format: &str| {
        d.format(format).to_string()
    });
    engine.register_fn("sub_days", |d: DateTime<Utc>, days: i64| {
        d - Duration::days(days)
    });
    if let Some(mapping) = state_mapping {
        for m in mapping {
            let result = if m.update.starts_with("$") {
                let s = &m.update[1..];
                let result: String = engine
                    .eval_with_scope(&mut scope, s)
                    .map_err(|_e| anyhow::anyhow!("failed to execute update state mapping"))?;
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

pub fn update_state(
    result: &Vec<ExportItem>,
    job: &JobCreation,
    ctx: Context,
) -> anyhow::Result<String> {
    update_state_core(result, &job.state, job.settings.state_mapping.as_ref(), ctx)
}
