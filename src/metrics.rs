use metrics::{counter, describe_counter, describe_histogram, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::time::Instant;

/// Initialize the Prometheus metrics recorder and return a handle for the metrics endpoint.
pub fn init_metrics() -> PrometheusHandle {
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    // Describe metrics
    describe_counter!(
        "apify_jobs_total",
        "Total number of Apify jobs processed"
    );
    describe_counter!(
        "apify_jobs_success_total",
        "Total number of successful Apify jobs"
    );
    describe_counter!(
        "apify_jobs_failed_total",
        "Total number of failed Apify jobs"
    );
    describe_histogram!(
        "apify_job_duration_seconds",
        "Duration of Apify job execution in seconds"
    );
    describe_counter!(
        "apify_api_requests_total",
        "Total number of API requests to Apify"
    );
    describe_histogram!(
        "apify_api_request_duration_seconds",
        "Duration of Apify API requests in seconds"
    );
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests received"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        "Duration of HTTP request handling in seconds"
    );

    handle
}

/// Record a job start.
pub fn record_job_started(actor_type: &str) {
    counter!("apify_jobs_total", "actor_type" => actor_type.to_string()).increment(1);
}

/// Record a job success.
pub fn record_job_success(actor_type: &str) {
    counter!("apify_jobs_success_total", "actor_type" => actor_type.to_string()).increment(1);
}

/// Record a job failure.
pub fn record_job_failed(actor_type: &str) {
    counter!("apify_jobs_failed_total", "actor_type" => actor_type.to_string()).increment(1);
}

/// Record job duration.
pub fn record_job_duration(actor_type: &str, duration_secs: f64) {
    histogram!("apify_job_duration_seconds", "actor_type" => actor_type.to_string())
        .record(duration_secs);
}

/// Record an API request to Apify.
pub fn record_api_request(endpoint: &str) {
    counter!("apify_api_requests_total", "endpoint" => endpoint.to_string()).increment(1);
}

/// Record API request duration.
pub fn record_api_duration(endpoint: &str, duration_secs: f64) {
    histogram!("apify_api_request_duration_seconds", "endpoint" => endpoint.to_string())
        .record(duration_secs);
}

/// Record an HTTP request.
pub fn record_http_request(method: &str, path: &str, status: u16) {
    counter!(
        "http_requests_total",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string()
    )
    .increment(1);
}

/// Record HTTP request duration.
pub fn record_http_duration(method: &str, path: &str, duration_secs: f64) {
    histogram!(
        "http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path.to_string()
    )
    .record(duration_secs);
}

/// A timer that records duration on drop.
pub struct Timer {
    start: Instant,
    actor_type: String,
}

impl Timer {
    pub fn new(actor_type: &str) -> Self {
        Self {
            start: Instant::now(),
            actor_type: actor_type.to_string(),
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Finish the timer and record the duration as a successful job.
    pub fn finish_success(self) {
        let duration = self.elapsed_secs();
        record_job_duration(&self.actor_type, duration);
        record_job_success(&self.actor_type);
    }

    /// Finish the timer and record the duration as a failed job.
    pub fn finish_failed(self) {
        let duration = self.elapsed_secs();
        record_job_duration(&self.actor_type, duration);
        record_job_failed(&self.actor_type);
    }
}
