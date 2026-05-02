use thiserror::Error;

/// Repository error reporting
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum RepoError {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("error details are redacted")]
    Redacted,
}
