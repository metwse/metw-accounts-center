use crate::repo::RepoError;
use thiserror::Error;

/// Service error reporting
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum ServiceError {
    #[error("repo: {0}")]
    Repo(#[from] RepoError),

    #[error("username taken")]
    UsernameTaken,

    #[error("email taken")]
    EmailTaken,

    #[error("unexcepted error: {0}")]
    UnexceptedError(&'static str),
}
