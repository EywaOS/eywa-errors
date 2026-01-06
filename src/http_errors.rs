//! HTTP error helper functions for common error patterns.

use super::app_error::{AppError, FieldError, ValidationErrors};

/// Create a not found error for a resource.
pub fn not_found(resource: &str, id: impl Into<String>) -> AppError {
    AppError::NotFound {
        resource: resource.to_string(),
        id: id.into(),
    }
}

/// Create a validation error for a single field.
pub fn validation_error(field: &str, message: impl Into<String>) -> AppError {
    AppError::ValidationField {
        field: field.to_string(),
        message: message.into(),
    }
}

/// Create a validation error with a specific code.
pub fn validation_error_with_code(field: &str, code: &str, message: impl Into<String>) -> AppError {
    let mut errors = ValidationErrors::new();
    errors.add(field, code, message);
    AppError::Validation(errors)
}

/// Create a validation error with the received value included.
pub fn validation_error_with_value(
    field: &str,
    code: &str,
    message: impl Into<String>,
    received: impl Into<serde_json::Value>,
) -> AppError {
    let mut errors = ValidationErrors::new();
    errors.add_with_value(field, code, message, received);
    AppError::Validation(errors)
}

/// Create an unauthorized error.
pub fn unauthorized() -> AppError {
    AppError::Unauthorized
}

/// Create a forbidden error.
pub fn forbidden(action: &str) -> AppError {
    AppError::Forbidden {
        action: action.to_string(),
    }
}

/// Create a conflict error.
pub fn conflict(message: impl Into<String>) -> AppError {
    AppError::Conflict {
        message: message.into(),
    }
}

/// Create an external service error.
pub fn external_service_error(service: &str) -> AppError {
    AppError::ExternalServiceError {
        service: service.to_string(),
    }
}

/// Create an internal server error.
pub fn internal_error(message: impl Into<String>) -> AppError {
    AppError::InternalServerError(message.into())
}

/// Create a bad request error.
pub fn bad_request(message: impl Into<String>) -> AppError {
    AppError::BadRequest(message.into())
}

/// Create a service unavailable error.
pub fn service_unavailable(message: impl Into<String>) -> AppError {
    AppError::ServiceUnavailable(message.into())
}

// =============================================================================
// Builder pattern for multiple validation errors
// =============================================================================

/// Builder for collecting multiple validation errors.
///
/// # Example
/// ```ignore
/// use eywa_errors::ValidationErrorBuilder;
///
/// let result = ValidationErrorBuilder::new()
///     .field("email", "invalid_format", "Must be a valid email")
///     .field("name", "too_short", "Must be at least 3 characters")
///     .build();
///
/// if let Err(app_error) = result {
///     return Err(app_error);
/// }
/// ```
pub struct ValidationErrorBuilder {
    errors: ValidationErrors,
}

impl ValidationErrorBuilder {
    /// Create a new validation error builder.
    pub fn new() -> Self {
        Self {
            errors: ValidationErrors::new(),
        }
    }

    /// Add a field error.
    pub fn field(mut self, field: &str, code: &str, message: impl Into<String>) -> Self {
        self.errors.add(field, code, message);
        self
    }

    /// Add a field error with the received value.
    pub fn field_with_value(
        mut self,
        field: &str,
        code: &str,
        message: impl Into<String>,
        received: impl Into<serde_json::Value>,
    ) -> Self {
        self.errors.add_with_value(field, code, message, received);
        self
    }

    /// Build the result. Returns Ok(()) if no errors, Err(AppError) otherwise.
    pub fn build(self) -> Result<(), AppError> {
        self.errors.into_result()
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for ValidationErrorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
