use std::{collections::HashMap, time::Duration};

use serde_json::Value;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    actors::ActorConfig,
    client::{ApiFyClient, State},
    dto::{ArbitraryActorJob, Data, ExportItem, JobCreation, KeyMapping, Response},
    extraction::extract_export_items,
    mapping_utils::{self, update_state, update_state_core},
    metrics::{record_job_started, Timer},
};

/// Prepares the request body by merging actor config with state mappings.
#[instrument(skip(actor_config, job), fields(actor_type = %actor_config.actor_type()))]
fn prepare_body(
    actor_config: &ActorConfig,
    job: &JobCreation,
) -> anyhow::Result<HashMap<String, Value>> {
    let mut body = actor_config
        .to_body()
        .map_err(|e| anyhow::anyhow!("Failed to serialize actor config: {}", e))?;

    let state: HashMap<String, Value> = serde_json::from_str(&job.state)?;
    if let Some(mapping) = &job.settings.state_mapping {
        for m in mapping {
            if let Some(v) = state.get(&m.from) {
                body.insert(m.to.clone(), v.clone());
            }
        }
    }
    Ok(body)
}

/// Starts an Apify actor job.
#[instrument(skip(client, actor_config, job), fields(actor_type = %actor_config.actor_type()))]
async fn start_job(
    client: &ApiFyClient,
    actor_config: &ActorConfig,
    job: &JobCreation,
) -> anyhow::Result<Data> {
    debug!("Validating actor configuration");
    actor_config
        .validate()
        .map_err(|e| anyhow::anyhow!("Invalid actor configuration: {}", e))?;

    let body = prepare_body(actor_config, job)?;
    info!("Starting Apify actor job");
    client.start_job(actor_config.actor_name(), &body).await
}

/// Polls for job completion and downloads results.
#[instrument(skip(client, job, data), fields(run_id = %data.id.0, dataset_id = %data.default_dataset_id.0))]
async fn fetch_results(
    client: &ApiFyClient,
    job: &JobCreation,
    data: Data,
) -> anyhow::Result<Vec<ExportItem>> {
    let mut poll_count = 0u32;
    loop {
        if let Ok(state) = client.check_completion(&data.id).await {
            match state {
                State::Running => {
                    poll_count += 1;
                    debug!(poll_count, "Job still running, waiting...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                State::Succeeded => {
                    info!(poll_count, "Job succeeded, downloading results");
                    let raw_data = client.download_results(&data.default_dataset_id).await?;
                    let items = extract_export_items(raw_data, &job.settings.key_mapping)?;
                    info!(item_count = items.len(), "Extracted export items");
                    return Ok(items);
                }
                State::Failed => {
                    error!("Actor job failed");
                    anyhow::bail!("Actor job failed");
                }
            }
        } else {
            warn!(poll_count, "Failed to check job completion status");
        }
    }
}

/// Validates that state mapping expressions are valid.
#[instrument(skip(job))]
pub fn validate_state_mapping(job: &JobCreation) -> anyhow::Result<()> {
    debug!("Validating state mapping expressions");
    update_state(&vec![], job, mapping_utils::Context::new())?;
    Ok(())
}

/// Runs a complete job: start, poll, fetch results, update state.
#[instrument(skip(job), fields(actor_type = %actor_type))]
pub async fn run_job(actor_type: &str, job: &JobCreation) -> anyhow::Result<Response> {
    // Parse the actor config from the path parameter and body
    let actor_config = ActorConfig::from_type_and_config(actor_type, job.settings.actor_config.clone())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    record_job_started(actor_type);
    let timer = Timer::new(actor_type);

    info!("Starting job execution");

    validate_state_mapping(job)?;

    let client = ApiFyClient::new(&job.settings.token);
    let data = match start_job(&client, &actor_config, job).await {
        Ok(data) => {
            info!(run_id = %data.id.0, "Job started successfully");
            data
        }
        Err(e) => {
            error!(error = %e, "Failed to start job");
            timer.finish_failed();
            return Err(e);
        }
    };

    let result = match fetch_results(&client, job, data).await {
        Ok(result) => result,
        Err(e) => {
            error!(error = %e, "Failed to fetch results");
            timer.finish_failed();
            return Err(e);
        }
    };

    let ctx = mapping_utils::Context::new();
    let state = match update_state(&result, job, ctx) {
        Ok(state) => state,
        Err(e) => {
            error!(error = %e, "Failed to update state");
            timer.finish_failed();
            return Err(e);
        }
    };

    timer.finish_success();
    info!(result_count = result.len(), "Job completed successfully");

    Ok(Response { state, result })
}

/// Prepares the request body for an arbitrary actor by merging input with state mappings.
#[instrument(skip(job), fields(actor_id = %job.settings.actor_id))]
fn prepare_arbitrary_body(job: &ArbitraryActorJob) -> anyhow::Result<HashMap<String, Value>> {
    let mut body: HashMap<String, Value> = match &job.settings.actor_input {
        Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => HashMap::new(),
    };

    let state: HashMap<String, Value> = serde_json::from_str(&job.state)?;
    if let Some(mapping) = &job.settings.state_mapping {
        for m in mapping {
            if let Some(v) = state.get(&m.from) {
                body.insert(m.to.clone(), v.clone());
            }
        }
    }
    Ok(body)
}

/// Polls for job completion and downloads results for arbitrary actor.
#[instrument(skip(client, key_mapping, data), fields(run_id = %data.id.0, dataset_id = %data.default_dataset_id.0))]
async fn fetch_arbitrary_results(
    client: &ApiFyClient,
    key_mapping: &[KeyMapping],
    data: Data,
) -> anyhow::Result<Vec<ExportItem>> {
    let mut poll_count = 0u32;
    loop {
        if let Ok(state) = client.check_completion(&data.id).await {
            match state {
                State::Running => {
                    poll_count += 1;
                    debug!(poll_count, "Job still running, waiting...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                State::Succeeded => {
                    info!(poll_count, "Job succeeded, downloading results");
                    let raw_data = client.download_results(&data.default_dataset_id).await?;
                    let items = extract_export_items(raw_data, key_mapping)?;
                    info!(item_count = items.len(), "Extracted export items");
                    return Ok(items);
                }
                State::Failed => {
                    error!("Actor job failed");
                    anyhow::bail!("Actor job failed");
                }
            }
        } else {
            warn!(poll_count, "Failed to check job completion status");
        }
    }
}

/// Runs an arbitrary Apify actor job.
#[instrument(skip(job), fields(actor_id = %job.settings.actor_id))]
pub async fn run_arbitrary_actor(job: &ArbitraryActorJob) -> anyhow::Result<Response> {
    let actor_id = &job.settings.actor_id;

    record_job_started(actor_id);
    let timer = Timer::new(actor_id);

    info!("Starting arbitrary actor job execution");

    // Validate state mapping
    debug!("Validating state mapping expressions");
    update_state_core(
        &vec![],
        &job.state,
        job.settings.state_mapping.as_ref(),
        mapping_utils::Context::new(),
    )?;

    let body = prepare_arbitrary_body(job)?;

    let client = ApiFyClient::new(&job.settings.token);
    let data = match client.start_job(actor_id, &body).await {
        Ok(data) => {
            info!(run_id = %data.id.0, "Job started successfully");
            data
        }
        Err(e) => {
            error!(error = %e, "Failed to start job");
            timer.finish_failed();
            return Err(e);
        }
    };

    let result = match fetch_arbitrary_results(&client, &job.settings.key_mapping, data).await {
        Ok(result) => result,
        Err(e) => {
            error!(error = %e, "Failed to fetch results");
            timer.finish_failed();
            return Err(e);
        }
    };

    let ctx = mapping_utils::Context::new();
    let state = match update_state_core(
        &result,
        &job.state,
        job.settings.state_mapping.as_ref(),
        ctx,
    ) {
        Ok(state) => state,
        Err(e) => {
            error!(error = %e, "Failed to update state");
            timer.finish_failed();
            return Err(e);
        }
    };

    timer.finish_success();
    info!(result_count = result.len(), "Arbitrary actor job completed successfully");

    Ok(Response { state, result })
}
