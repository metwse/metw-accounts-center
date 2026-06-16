use crate::service::ServiceError;
use thiserror::Error;

/// Handler error reporting.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum HandlerError {
    #[error("{0}")]
    Service(#[from] ServiceError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("unexpected error: {0}")]
    UnexpectedError(&'static str),
}
