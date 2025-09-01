use serde::Deserialize;
use serde_json::Value;

use crate::dto::{Data, Root, RunId};

pub struct ApiFyClient {
    token: String,
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
        Self {
            token: token.to_string(),
            client: reqwest::Client::default(),
        }
    }

    pub async fn start_job(&self, actor: &str, body: &Value) -> anyhow::Result<Data> {
        let url = format!(
            "https://api.apify.com/v2/acts/{}/runs?token={}",
            actor, &self.token
        );
        let resp: Root = self
            .client
            .post(url)
            .json(body)
            .send()
            .await?
            .json()
            .await?;
        return Ok(resp.data);
    }

    pub async fn download_results(
        &self,
        DatasetId(dataset_id): &DatasetId,
    ) -> anyhow::Result<Vec<Value>> {
        let url = format!(
            "https://api.apify.com/v2/datasets/{}/items?token={}",
            dataset_id, &self.token
        );
        let resp: Vec<Value> = self.client.get(url).send().await?.json().await?;

        Ok(resp)
    }

    pub async fn check_completion(&self, RunId(run_id): &RunId) -> anyhow::Result<State> {
        let url = format!(
            "https://api.apify.com/v2/actor-runs/{}?token={}",
            run_id, &self.token
        );
        let resp: StateDto = self.client.get(url).send().await?.json().await?;

        Ok(match resp.data.status.as_str() {
            "SUCCEEDED" => State::Succeeded,
            "RUNNING" => State::Running,
            _ => State::Failed,
        })
    }
}

pub enum State {
    Running,
    Succeeded,
    Failed,
}
