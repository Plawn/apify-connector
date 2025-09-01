use anyhow::{Context, Result};
use apify_connector::{
    client::{ApiFyClient, State},
    dto::Data,
    utils::AppError,
};
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::Read, time::Duration};

#[derive(Debug, Deserialize)]
pub enum DataKind {
    Date { format: String },
    String,
}

#[derive(Debug, Deserialize)]
pub struct KeyMapping {
    from: String,
    to: String,
    kind: DataKind,
}

#[derive(Debug, Deserialize)]
struct Settings {
    actor: String,
    token: String,
    body: Value,
    key_mapping: Vec<KeyMapping>, // todo make real
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

/// job with all settings and state
#[derive(Debug, Deserialize)]
struct JobCreation {
    settings: Settings,
    /// Json encoded state
    state: String,
}

async fn start_job(client: &ApiFyClient, job: &JobCreation) -> anyhow::Result<Data> {
    client
        .start_job(&job.settings.actor, &job.settings.body)
        .await
}

async fn fetch_results(
    client: &ApiFyClient,
    key_mapping: &Vec<KeyMapping>,
    j: Data,
) -> anyhow::Result<Vec<ExportItem>> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt; // for write_all()
    loop {
        if let Ok(r) = client.check_completion(&j.id).await {
            match r {
                State::Running => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                State::Succeeded => {
                    let data = client.download_results(&j.defaultDatasetId).await?;
                    let mut file = File::create("foo.json").await?;
                    let d = serde_json::to_string_pretty(&data).unwrap();
                    file.write_all(d.as_bytes()).await?;

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

#[derive(Serialize, Clone, Debug)]
pub struct ExportItem {
    id: Option<String>,
    content: String,
    #[serde(with = "jackson")]
    date: DateTime<Utc>,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Debug)]
struct Response {
    state: String,
    result: Vec<ExportItem>,
}

async fn handle_job(
    Json(job): Json<JobCreation>,
) -> Result<(StatusCode, Json<Response>), AppError> {
    // TODO: check input, ensure, key mapping has id, date and content
    let client = ApiFyClient::new(&job.settings.token);
    match start_job(&client, &job).await {
        Ok(j) => match fetch_results(&client, &job.settings.key_mapping, j).await {
            Ok(result) => Ok((
                StatusCode::OK,
                Json(Response {
                    // TODO: finish here
                    // handle date
                    state: "{}".to_string(),
                    result,
                }),
            )),
            Err(e) => Err(AppError::from(e.to_string())),
        },
        Err(e) => Err(AppError::from(e.to_string())),
    }
}
