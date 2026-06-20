use thiserror::Error;

/// Repository error reporting
///
/// The internal errors must be redacted before returning to the public.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum RepoError {
    #[error("internal error: {0}")]
    Internal(&'static str),

    #[error("internal sqlx error: {0}")]
    InternalSqlx(#[from] sqlx::error::Error),

    #[error("internal redis error: {0}")]
    InternalRedis(#[from] redis::RedisError),
}
