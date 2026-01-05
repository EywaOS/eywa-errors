mod app_error;
mod http_errors;

pub use app_error::{AppError, ErrorResponse};
pub use http_errors::*;
pub type Result<T> = std::result::Result<T, AppError>;
