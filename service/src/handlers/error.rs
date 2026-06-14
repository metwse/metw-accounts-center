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

    #[error("unexcepted error: {0}")]
    UnexceptedError(&'static str),
}
