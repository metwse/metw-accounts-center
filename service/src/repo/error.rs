use thiserror::Error;

/// Repository error reporting
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum RepoError {
    // The internal errors must be redacted before returning to the public.

    #[error("internal error: {0}")]
    Internal(&'static str),

    #[error("internal sqlx error: {0}")]
    InternalSqlx(#[from] sqlx::error::Error),

    #[error("internal redis error: {0}")]
    InternalRedis(#[from] redis::RedisError),

    #[error("error details are redacted")]
    Redacted,
}
