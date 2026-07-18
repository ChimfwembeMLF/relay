use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("country not enabled")]
    CountryNotEnabled,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("gateway error: {0}")]
    Gateway(String),
    #[error("invoice invalid: {0}")]
    InvoiceInvalid(String),
    #[error("payload too large: {0}")]
    PayloadTooLarge(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error, message) = match &self {
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error", msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg.clone()),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Invalid or missing API key".into(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "Access denied for this resource".into(),
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", "Resource not found".into()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::InsufficientBalance => (
                StatusCode::PAYMENT_REQUIRED,
                "insufficient_balance",
                "Insufficient wallet balance".into(),
            ),
            AppError::CountryNotEnabled => (
                StatusCode::FORBIDDEN,
                "country_not_enabled",
                "Country not enabled for this system".into(),
            ),
            AppError::Database(e) => {
                tracing::error!(error = %e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "Database operation failed".into(),
                )
            }
            AppError::Gateway(msg) => (StatusCode::BAD_GATEWAY, "gateway_error", msg.clone()),
            AppError::InvoiceInvalid(msg) => (StatusCode::PAYMENT_REQUIRED, "invoice_invalid", msg.clone()),
            AppError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large", msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg.clone()),
        };

        (
            status,
            Json(ErrorBody {
                error: error.into(),
                message,
            }),
        )
            .into_response()
    }
}
