use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use std::error::Error;
use std::fmt;

/// JSON structure for error responses
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
}

/// Custom application error type
#[derive(Debug)]
pub struct AppError {
    message: String,
}

impl AppError {
    pub fn from(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (status, body).into_response()
    }
}
