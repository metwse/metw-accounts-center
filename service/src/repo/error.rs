use thiserror::Error;

/// Repository error reporting
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum RepoError {
    /// DO NOT EXPOSE PUBLIC
    #[error("internal error: {0}")]
    Internal(&'static str),

    #[error("error details are redacted")]
    Redacted,
}
