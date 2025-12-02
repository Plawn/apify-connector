use apify_connector::handlers::{get_actor_schema, handle_arbitrary_actor, handle_job, list_actors};
use apify_connector::metrics::init_metrics;
use axum::{
    Router,
    routing::{get, post},
};
use metrics_exporter_prometheus::PrometheusHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// GET /metrics - Prometheus metrics endpoint
async fn metrics_handler(
    axum::extract::State(handle): axum::extract::State<PrometheusHandle>,
) -> String {
    handle.render()
}

#[tokio::main]
async fn main() {
    // Initialize tracing with env filter support
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apify_connector=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize Prometheus metrics
    let metrics_handle = init_metrics();

    let port = 8000;
    let app = Router::new()
        .route("/actors", get(list_actors))
        .route("/actors/{actor_type}", get(get_actor_schema))
        .route("/run", post(handle_arbitrary_actor))
        .route("/{actor_type}", post(handle_job))
        .route("/metrics", get(metrics_handler))
        .with_state(metrics_handle);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    tracing::info!("Listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
