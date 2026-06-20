use crate::service::ServiceError;
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

/// Handler error reporting.
#[derive(Error, Debug, Serialize)]
#[allow(missing_docs)]
pub enum HandlerError {
    #[error("{0}")]
    Service(#[from] ServiceError),

    #[error("validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("unauthorized")]
    Unauthorized,

    #[error("unexpected error: {0}")]
    UnexpectedError(&'static str),
}
