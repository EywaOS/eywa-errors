mod app_error;
mod http_errors;

pub use app_error::{
    AppError, CURRENT_REQUEST_ID, FieldError, ProblemDetails, ValidationErrors, get_request_id,
    set_request_id,
};

#[allow(deprecated)]
pub use app_error::ErrorResponse;

pub use http_errors::*;

pub type Result<T> = std::result::Result<T, AppError>;
