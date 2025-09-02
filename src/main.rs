use anyhow::{Context, Result};
use apify_connector::{
    client::{ApiFyClient, State},
    dto::{Data, DataKind, ExportItem, JobCreation, KeyMapping, Response},
    mapping_utils::{self, update_state},
    web_utils::AppError,
};
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use chrono::{NaiveDate, TimeZone, Utc};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};

fn prepapre_body(job: &JobCreation) -> anyhow::Result<HashMap<String, Value>> {
    let mut body = job.settings.body.clone();

    if let Some(mapping) = &job.settings.state_mapping {
        for m in mapping {
            if let Some(v) = job.state.get(&m.from) {
                body.insert(m.to.clone(), v.clone());
            }
        }
    }
    Ok(body)
}

async fn start_job(client: &ApiFyClient, job: &JobCreation) -> anyhow::Result<Data> {
    let body = prepapre_body(job)?;
    // client.start_job(&job.settings.actor, &body).await
    println!("prepapred body: {:?}", body);
    todo!()
}

async fn fetch_results(
    client: &ApiFyClient,
    key_mapping: &Vec<KeyMapping>,
    j: Data,
) -> anyhow::Result<Vec<ExportItem>> {
    loop {
        if let Ok(r) = client.check_completion(&j.id).await {
            match r {
                State::Running => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                State::Succeeded => {
                    let data = client.download_results(&j.defaultDatasetId).await?;
                    return Ok(extract_export_items(data, key_mapping)?);
                }
                State::Failed => anyhow::bail!("failed to run"),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/", post(handle_job));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// // basic handler that responds with a static string
async fn root() -> &'static str {
    "OK"
}

/// Extracts a Vec<ExportItem> from a serde_json::Value that is a JSON array.
pub fn extract_export_items(
    data: Vec<Value>,
    key_mappings: &[KeyMapping],
) -> anyhow::Result<Vec<ExportItem>> {
    println!("count: {}", data.len());

    let e: Vec<_> = data
        .iter()
        .filter_map(
            |item_value| match extract_single_export_item(item_value, key_mappings) {
                Ok(r) => Some(r),
                Err(_) => {
                    // eprintln!("data failed: {} {}", item_value, e);
                    None
                }
            },
        )
        .collect();
    Ok(e)
}

/// Extracts a single ExportItem from a serde_json::Value that is a JSON object.
fn extract_single_export_item(
    data: &Value,
    key_mappings: &[KeyMapping],
) -> anyhow::Result<ExportItem> {
    let mut id = None;
    let mut content = None;
    let mut date = None;
    let mut metadata = HashMap::new();

    // Ensure the item is a JSON object.
    let map = data
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Item in array is not a JSON object"))?;

    let mut mapped_keys = std::collections::HashSet::new();

    for mapping in key_mappings {
        if let Some(value) = map.get(&mapping.from) {
            mapped_keys.insert(&mapping.from);

            match mapping.to.as_str() {
                "id" => id = value.as_str().map(String::from),
                "content" => content = value.as_str().map(String::from),
                "date" => {
                    if let DataKind::Date { format } = &mapping.kind {
                        if let Some(s) = value.as_str() {
                            // Add context to the error if date parsing fails
                            let parsed_date =
                                NaiveDate::parse_from_str(s, format).with_context(|| {
                                    format!("Failed to parse date '{}' with format '{}'", s, format)
                                })?;
                            date = Some(
                                Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap()),
                            );
                        }
                    }
                }
                // Any other explicitly mapped key goes into metadata
                _ => {
                    if let Some(s) = value.as_str() {
                        metadata.insert(mapping.to.clone(), s.to_string());
                    }
                }
            }
        }
    }

    // Collect any unmapped fields from the source object into metadata
    for (key, value) in map {
        if !mapped_keys.contains(key) {
            if let Some(s) = value.as_str() {
                metadata.insert(key.clone(), s.to_string());
            }
        }
    }

    Ok(ExportItem {
        id,
        content: content.ok_or_else(|| anyhow::anyhow!("Missing 'content' field in an item"))?,
        date: date.ok_or_else(|| anyhow::anyhow!("Missing or invalid 'date' field in an item"))?,
        metadata,
    })
}

async fn handle_job(
    Json(job): Json<JobCreation>,
) -> Result<(StatusCode, Json<Response>), AppError> {
    // TODO: check input, ensure, key mapping has id, date and content
    let client = ApiFyClient::new(&job.settings.token);
    let ctx = mapping_utils::Context { start: Utc::now() };
    match start_job(&client, &job).await {
        Ok(j) => match fetch_results(&client, &job.settings.key_mapping, j).await {
            Ok(result) => {
                let state =
                    update_state(&result, &job, &ctx).map_err(|e| AppError::from(e.to_string()))?;
                Ok((StatusCode::OK, Json(Response { state, result })))
            }
            Err(e) => Err(AppError::from(e.to_string())),
        },
        Err(e) => Err(AppError::from(e.to_string())),
    }
}
