use std::collections::HashMap;
use std::time::Instant;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info, instrument};

use crate::actors::ActorConfig;
use crate::dto::{Data, Root, RunId};
use crate::metrics::{record_api_duration, record_api_request};

const APIFY_API_BASE: &str = "https://api.apify.com/v2";

pub struct ApiFyClient {
    client: reqwest::Client,
}

#[derive(Deserialize, Debug)]
pub struct DatasetId(pub String);

#[derive(Deserialize, Debug)]
pub struct StateData {
    status: String,
}

#[derive(Deserialize, Debug)]
pub struct StateDto {
    data: StateData,
}

impl ApiFyClient {
    pub fn new(token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .expect("Invalid token format"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    #[instrument(skip(self, body), fields(actor = %actor))]
    pub async fn start_job(
        &self,
        actor: &str,
        body: &HashMap<String, Value>,
    ) -> anyhow::Result<Data> {
        let start = Instant::now();
        record_api_request("start_job");

        let url = format!("{}/acts/{}/runs", APIFY_API_BASE, actor);
        debug!("Sending start job request");
        let resp: Root = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await?
            .json()
            .await?;

        record_api_duration("start_job", start.elapsed().as_secs_f64());
        info!(run_id = %resp.data.id.0, "Job started");
        Ok(resp.data)
    }

    /// Start a job using a typed actor configuration
    pub async fn start_job_typed(&self, config: &ActorConfig) -> anyhow::Result<Data> {
        let body = config
            .to_body()
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
        self.start_job(config.actor_name(), &body).await
    }

    #[instrument(skip(self), fields(dataset_id = %dataset_id))]
    pub async fn download_results(
        &self,
        DatasetId(dataset_id): &DatasetId,
    ) -> anyhow::Result<Vec<Value>> {
        let start = Instant::now();
        record_api_request("download_results");

        let url = format!("{}/datasets/{}/items", APIFY_API_BASE, dataset_id);
        debug!("Downloading dataset results");
        let resp: Vec<Value> = self.client.get(&url).send().await?.json().await?;

        record_api_duration("download_results", start.elapsed().as_secs_f64());
        info!(item_count = resp.len(), "Downloaded results");
        Ok(resp)
    }

    #[instrument(skip(self), fields(run_id = %run_id))]
    pub async fn check_completion(&self, RunId(run_id): &RunId) -> anyhow::Result<State> {
        let start = Instant::now();
        record_api_request("check_completion");

        let url = format!("{}/actor-runs/{}", APIFY_API_BASE, run_id);
        let resp: StateDto = self.client.get(&url).send().await?.json().await?;

        record_api_duration("check_completion", start.elapsed().as_secs_f64());
        let state = match resp.data.status.as_str() {
            "SUCCEEDED" => State::Succeeded,
            "RUNNING" => State::Running,
            _ => State::Failed,
        };
        debug!(status = %resp.data.status, "Checked job status");
        Ok(state)
    }
}

pub enum State {
    Running,
    Succeeded,
    Failed,
}
