mod app_error;
mod http_errors;

pub use app_error::{get_request_id, AppError, ErrorResponse, CURRENT_REQUEST_ID};
pub use http_errors::*;
pub type Result<T> = std::result::Result<T, AppError>;
