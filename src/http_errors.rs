use super::app_error::{AppError, ValidationError};

pub fn not_found(resource: &str, id: impl Into<String>) -> AppError {
    AppError::NotFound {
        resource: resource.to_string(),
        id: id.into(),
    }
}

pub fn validation_error(field: &str, message: impl Into<String>) -> AppError {
    AppError::ValidationError {
        field: field.to_string(),
        message: message.into(),
    }
}

pub fn validation_error_from(error: &ValidationError) -> AppError {
    match error {
        ValidationError::Required { field } => AppError::ValidationError {
            field: field.clone(),
            message: format!("Field '{}' is required", field),
        },
        ValidationError::InvalidEmail { email } => AppError::ValidationError {
            field: "email".to_string(),
            message: format!("Invalid email format: {}", email),
        },
        ValidationError::PasswordTooWeak => AppError::ValidationError {
            field: "password".to_string(),
            message: "Password is too weak. Minimum 8 characters required".to_string(),
        },
        ValidationError::InvalidUrl { url } => AppError::ValidationError {
            field: "url".to_string(),
            message: format!("Invalid URL: {}", url),
        },
        ValidationError::TooLarge { field, max } => AppError::ValidationError {
            field: field.clone(),
            message: format!("Field '{}' is too large. Maximum size: {}", field, max),
        },
    }
}

pub fn unauthorized() -> AppError {
    AppError::Unauthorized
}

pub fn forbidden(action: &str) -> AppError {
    AppError::Forbidden {
        action: action.to_string(),
    }
}

pub fn conflict(message: impl Into<String>) -> AppError {
    AppError::Conflict {
        message: message.into(),
    }
}

pub fn external_service_error(service: &str) -> AppError {
    AppError::ExternalServiceError {
        service: service.to_string(),
    }
}

pub fn internal_error(message: impl Into<String>) -> AppError {
    AppError::InternalServerError(message.into())
}
