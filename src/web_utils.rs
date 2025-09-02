use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

// Définir une structure pour les réponses d'erreur
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
}

// Définir un type d'erreur personnalisé
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

// Implémenter la conversion en réponse HTTP pour AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (status, body).into_response()
    }
}

// S
