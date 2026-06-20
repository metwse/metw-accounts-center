use crate::repo::RepoError;
use thiserror::Error;

/// Service error reporting.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum ServiceError {
    #[error("repo: {0}")]
    Repo(#[from] RepoError),

    #[error("username taken")]
    UsernameTaken,

    #[error("email taken")]
    EmailTaken,

    #[error("cannot delete primary email")]
    CannotDeletePrimaryEmail,

    #[error("account not found")]
    AccountNotFound,

    #[error("email not found")]
    EmailNotFound,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("add email failed")]
    AddEmailFailed,

    #[error("change primary email failed")]
    ChangePrimaryEmailFailed,

    #[error("signup complete failed")]
    SignupCompleteFailed,

    #[error("invalid jwt")]
    InvalidJwt,

    #[error("token revoked")]
    TokenRevoked,
}
