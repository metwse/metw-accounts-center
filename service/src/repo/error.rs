use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};
use thiserror::Error;

/// Repository error reporting
///
/// The internal errors must be redacted before returning to the public.
#[serde_as]
#[derive(Error, Debug, Serialize)]
#[allow(missing_docs)]
pub enum RepoError {
    #[error("internal error: {0}")]
    Internal(&'static str),

    #[error("internal sqlx error: {0}")]
    InternalSqlx(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        sqlx::error::Error,
    ),

    #[error("internal redis error: {0}")]
    InternalRedis(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        redis::RedisError,
    ),
}
