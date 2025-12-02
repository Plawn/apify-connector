use std::time::Instant;

use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use tracing::{error, info, instrument};

use crate::{
    actors::{get_actor_metadata, list_available_actors, ActorMetadata},
    dto::{ArbitraryActorJob, JobCreation, Response},
    job::{run_arbitrary_actor, run_job},
    metrics::{record_http_duration, record_http_request},
    web_utils::AppError,
};

/// POST /:actor_type - Execute an Apify actor job
#[instrument(skip(job), fields(actor_type = %actor_type))]
pub async fn handle_job(
    Path(actor_type): Path<String>,
    Json(job): Json<JobCreation>,
) -> Result<(StatusCode, Json<Response>), AppError> {
    let start = Instant::now();
    info!("Received job request");

    let response = run_job(&actor_type, &job).await.map_err(|e| {
        error!(error = %e, "Job execution failed");
        record_http_request("POST", &format!("/{}", actor_type), 502);
        record_http_duration("POST", &format!("/{}", actor_type), start.elapsed().as_secs_f64());
        AppError::bad_gateway(e.to_string())
    })?;

    record_http_request("POST", &format!("/{}", actor_type), 200);
    record_http_duration("POST", &format!("/{}", actor_type), start.elapsed().as_secs_f64());
    info!(result_count = response.result.len(), "Job completed");

    Ok((StatusCode::OK, Json(response)))
}

/// GET /actors - List all available actors with their schemas
#[instrument]
pub async fn list_actors() -> Json<Vec<ActorMetadata>> {
    let start = Instant::now();
    let actors = list_available_actors();
    record_http_request("GET", "/actors", 200);
    record_http_duration("GET", "/actors", start.elapsed().as_secs_f64());
    info!(actor_count = actors.len(), "Listed available actors");
    Json(actors)
}

/// GET /actors/:actor_type - Get schema for a specific actor
#[instrument]
pub async fn get_actor_schema(
    Path(actor_type): Path<String>,
) -> Result<Json<ActorMetadata>, AppError> {
    let start = Instant::now();

    let metadata = get_actor_metadata(&actor_type).ok_or_else(|| {
        record_http_request("GET", &format!("/actors/{}", actor_type), 404);
        record_http_duration("GET", &format!("/actors/{}", actor_type), start.elapsed().as_secs_f64());
        AppError::not_found(format!("Unknown actor type: {}", actor_type))
    })?;

    record_http_request("GET", &format!("/actors/{}", actor_type), 200);
    record_http_duration("GET", &format!("/actors/{}", actor_type), start.elapsed().as_secs_f64());
    info!("Retrieved actor schema");

    Ok(Json(metadata))
}

/// POST /run - Execute an arbitrary Apify actor job
#[instrument(skip(job), fields(actor_id = %job.settings.actor_id))]
pub async fn handle_arbitrary_actor(
    Json(job): Json<ArbitraryActorJob>,
) -> Result<(StatusCode, Json<Response>), AppError> {
    let start = Instant::now();
    let actor_id = job.settings.actor_id.clone();
    info!("Received arbitrary actor job request");

    let response = run_arbitrary_actor(&job).await.map_err(|e| {
        error!(error = %e, "Arbitrary actor job execution failed");
        record_http_request("POST", "/run", 502);
        record_http_duration("POST", "/run", start.elapsed().as_secs_f64());
        AppError::bad_gateway(e.to_string())
    })?;

    record_http_request("POST", "/run", 200);
    record_http_duration("POST", "/run", start.elapsed().as_secs_f64());
    info!(actor_id = %actor_id, result_count = response.result.len(), "Arbitrary actor job completed");

    Ok((StatusCode::OK, Json(response)))
}
