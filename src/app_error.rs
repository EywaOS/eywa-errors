use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

tokio::task_local! {
    /// Task-local storage for the current request ID.
    /// Set by the request_context middleware in eywa-axum.
    pub static CURRENT_REQUEST_ID: Uuid;
}

/// Sets the current request ID for this task scope.
/// Called by eywa-axum's request_context middleware.
pub fn set_request_id<F, R>(request_id: Uuid, f: F) -> R
where
    F: FnOnce() -> R,
{
    CURRENT_REQUEST_ID.sync_scope(request_id, f)
}

/// Gets the current request ID if set, otherwise generates a new one.
pub fn get_request_id() -> Uuid {
    CURRENT_REQUEST_ID
        .try_with(|id| *id)
        .unwrap_or_else(|_| Uuid::new_v4())
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found: {resource} with id: {id}")]
    NotFound { resource: String, id: String },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {action}")]
    Forbidden { action: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("External service error: {service}")]
    ExternalServiceError { service: String },

    #[error("Internal error: {0}")]
    InternalServerError(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: String,
    pub request_id: String,
    pub timestamp: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, code) = match &self {
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, "Not Found", "NOT_FOUND"),
            AppError::ValidationError { .. } => {
                (StatusCode::BAD_REQUEST, "Bad Request", "VALIDATION_ERROR")
            }
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad Request", "BAD_REQUEST"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized", "UNAUTHORIZED"),
            AppError::Forbidden { .. } => (StatusCode::FORBIDDEN, "Forbidden", "FORBIDDEN"),
            AppError::Conflict { .. } => (StatusCode::CONFLICT, "Conflict", "CONFLICT"),
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "DATABASE_ERROR",
            ),
            AppError::ConfigError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration Error",
                "CONFIG_ERROR",
            ),
            AppError::ExternalServiceError { .. } => (
                StatusCode::BAD_GATEWAY,
                "Bad Gateway",
                "EXTERNAL_SERVICE_ERROR",
            ),
            AppError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "INTERNAL_ERROR",
            ),
            AppError::ServiceUnavailable(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "SERVICE_UNAVAILABLE",
            ),
        };

        // Use request_id from task-local storage (set by middleware) or generate new
        let request_id = get_request_id();

        let response = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            code: code.to_string(),
            request_id: request_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        tracing::error!(
            status = %status,
            code = %code,
            message = %self.to_string(),
            request_id = %response.request_id,
            "Error occurred"
        );

        (status, Json(response)).into_response()
    }
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Required field: {field}")]
    Required { field: String },

    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },

    #[error("Password too weak")]
    PasswordTooWeak,

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[error("Value too large: max {max}")]
    TooLarge { field: String, max: u64 },
}

impl ValidationError {
    pub fn required_field(field: &str) -> Self {
        Self::Required {
            field: field.to_string(),
        }
    }

    pub fn invalid_email_field(email: &str) -> Self {
        Self::InvalidEmail {
            email: email.to_string(),
        }
    }

    pub fn password_weak() -> Self {
        Self::PasswordTooWeak
    }

    pub fn invalid_url_field(url: &str) -> Self {
        Self::InvalidUrl {
            url: url.to_string(),
        }
    }

    pub fn too_large_field(field: &str, max: u64) -> Self {
        Self::TooLarge {
            field: field.to_string(),
            max,
        }
    }
}

pub mod prelude {
    pub use super::{AppError, ValidationError};
}
