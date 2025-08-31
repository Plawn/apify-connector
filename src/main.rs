use std::collections::HashMap;

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyMapping {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    body: Value,
    key_mapping: Vec<KeyMapping>, // todo make real
}
#[derive(Serialize, Clone, Debug)]
pub struct ExportItem {
    id: String,
    content: Option<String>,
    #[serde(with = "jackson")]
    date: DateTime<Utc>,
    metadata: HashMap<String, String>,
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
#[derive(Debug, Deserialize, Serialize)]
struct Job {
    actor: String,
    token: String,
    settings: Settings,
    state: String,
}

async fn start_job(job: &Job) -> String {
    // curl "https://api.apify.com/v2/acts/Hvp4YfFGyLM635Q2F/runs?token=$API_TOKEN" \

    let url = format!(
        "https://api.apify.com/v2/acts/{}/runs?token={}",
        job.actor, job.token
    );
    let client = reqwest::Client::default();
    let mut builder = client.post(url);
    builder = builder.json(&job.settings.body);

    let req: Value = builder.send().await.unwrap().json().await.unwrap();
    println!("resp: {}", &req);
    "Ok".to_string()
}

async fn fetch_results() -> Res {
    // handle result mapping with key mapppings
    Res {}
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

#[derive(Debug, Deserialize, Serialize)]
struct Res {}

async fn handle_job(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(job): Json<Job>,
) -> (StatusCode, Json<Res>) {
    // TODO implem
    let _ = start_job(&job).await;
    let res = fetch_results().await;
    (StatusCode::CREATED, Json(res))
}
