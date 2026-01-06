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

// =============================================================================
// RFC 7807 Problem Details
// =============================================================================

/// RFC 7807 Problem Details response format.
///
/// This provides a standardized way to carry machine-readable details of errors
/// in HTTP responses. See: https://tools.ietf.org/html/rfc7807
///
/// # Example Response
/// ```json
/// {
///   "type": "https://api.example.com/errors/validation-error",
///   "title": "Validation Error",
///   "status": 400,
///   "detail": "The 'email' field must be a valid email address",
///   "instance": "/users/abc123",
///   "request_id": "550e8400-e29b-41d4-a716-446655440000",
///   "timestamp": "2026-01-06T14:17:00Z",
///   "errors": [
///     { "field": "email", "code": "invalid_format", "message": "Must be a valid email" }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProblemDetails {
    /// URI reference that identifies the problem type.
    /// When dereferenced, should provide human-readable documentation.
    #[serde(rename = "type")]
    pub error_type: String,

    /// Short, human-readable summary of the problem type.
    pub title: String,

    /// HTTP status code.
    pub status: u16,

    /// Human-readable explanation specific to this occurrence of the problem.
    pub detail: String,

    /// URI reference that identifies the specific occurrence of the problem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Unique request identifier for tracing.
    pub request_id: String,

    /// ISO 8601 timestamp of when the error occurred.
    pub timestamp: String,

    /// Field-level validation errors (if applicable).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<FieldError>,
}

/// Field-level error for validation failures.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FieldError {
    /// The field that caused the error.
    pub field: String,

    /// Machine-readable error code.
    pub code: String,

    /// Human-readable error message.
    pub message: String,

    /// The value that was received (for debugging).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received: Option<serde_json::Value>,
}

impl FieldError {
    /// Create a new field error.
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
            received: None,
        }
    }

    /// Create a new field error with the received value.
    pub fn with_received(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        received: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
            received: Some(received.into()),
        }
    }
}

// =============================================================================
// AppError
// =============================================================================

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found: {resource} with id: {id}")]
    NotFound { resource: String, id: String },

    #[error("Validation error: {0}")]
    Validation(ValidationErrors),

    #[error("Validation error: {field} - {message}")]
    ValidationField { field: String, message: String },

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

impl AppError {
    /// Get the error type URI for this error.
    fn error_type_uri(&self) -> &'static str {
        match self {
            AppError::NotFound { .. } => "https://errors.eywa.dev/not-found",
            AppError::Validation(_) | AppError::ValidationField { .. } => {
                "https://errors.eywa.dev/validation-error"
            }
            AppError::Unauthorized => "https://errors.eywa.dev/unauthorized",
            AppError::Forbidden { .. } => "https://errors.eywa.dev/forbidden",
            AppError::Conflict { .. } => "https://errors.eywa.dev/conflict",
            AppError::DatabaseError(_) => "https://errors.eywa.dev/database-error",
            AppError::ConfigError(_) => "https://errors.eywa.dev/config-error",
            AppError::ExternalServiceError { .. } => {
                "https://errors.eywa.dev/external-service-error"
            }
            AppError::InternalServerError(_) => "https://errors.eywa.dev/internal-error",
            AppError::BadRequest(_) => "https://errors.eywa.dev/bad-request",
            AppError::ServiceUnavailable(_) => "https://errors.eywa.dev/service-unavailable",
        }
    }

    /// Get the HTTP status code and title for this error.
    fn status_and_title(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, "Not Found"),
            AppError::Validation(_) | AppError::ValidationField { .. } => {
                (StatusCode::BAD_REQUEST, "Validation Error")
            }
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad Request"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Forbidden { .. } => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::Conflict { .. } => (StatusCode::CONFLICT, "Conflict"),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
            AppError::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration Error"),
            AppError::ExternalServiceError { .. } => {
                (StatusCode::BAD_GATEWAY, "External Service Error")
            }
            AppError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            AppError::ServiceUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable")
            }
        }
    }

    /// Convert to ProblemDetails.
    pub fn to_problem_details(&self) -> ProblemDetails {
        let (status, title) = self.status_and_title();
        let request_id = get_request_id();

        let errors = match self {
            AppError::Validation(v) => v.errors.clone(),
            AppError::ValidationField { field, message } => {
                vec![FieldError::new(field, "validation_error", message)]
            }
            _ => Vec::new(),
        };

        ProblemDetails {
            error_type: self.error_type_uri().to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            detail: self.to_string(),
            instance: None,
            request_id: request_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            errors,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, _) = self.status_and_title();
        let problem = self.to_problem_details();

        tracing::error!(
            status = %status,
            error_type = %problem.error_type,
            detail = %problem.detail,
            request_id = %problem.request_id,
            "Error occurred"
        );

        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
            Json(problem),
        )
            .into_response()
    }
}

// =============================================================================
// ValidationErrors Collection
// =============================================================================

/// Collection of validation errors for multiple fields.
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    pub errors: Vec<FieldError>,
}

impl ValidationErrors {
    /// Create a new empty validation errors collection.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add a field error.
    pub fn add(
        &mut self,
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.errors.push(FieldError::new(field, code, message));
    }

    /// Add a field error with the received value.
    pub fn add_with_value(
        &mut self,
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        received: impl Into<serde_json::Value>,
    ) {
        self.errors
            .push(FieldError::with_received(field, code, message, received));
    }

    /// Check if there are any errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Convert to AppError if there are errors, otherwise Ok(()).
    pub fn into_result(self) -> Result<(), AppError> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(self))
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages: Vec<_> = self
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        write!(f, "{}", messages.join(", "))
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        AppError::Validation(errors)
    }
}

// =============================================================================
// Legacy Compatibility (deprecated, will be removed)
// =============================================================================

/// Legacy error response format.
///
/// **Deprecated**: Use `ProblemDetails` instead.
#[deprecated(since = "0.2.0", note = "Use ProblemDetails instead")]
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: String,
    pub request_id: String,
    pub timestamp: String,
}

// =============================================================================
// Prelude
// =============================================================================

pub mod prelude {
    pub use super::{AppError, FieldError, ProblemDetails, ValidationErrors};
}
