use crate::repo::RepoError;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;

/// Service error reporting.
#[derive(Error, Debug, Serialize)]
#[allow(missing_docs)]
pub enum ServiceError {
    #[error("repo: {0}")]
    Repo(#[from] RepoError),

    #[error("username taken")]
    UsernameTaken,

    #[error("email taken")]
    EmailTaken,

    #[error("cannot delete primary email or email not found")]
    CannotDeletePrimaryEmailOrEmailNotFound,

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

    #[error("email limited: {0:?}")]
    EmailLimited(Duration),

    #[error("cannot have more than {0} emails")]
    TooManyEmails(usize),

    #[error("application not found")]
    ApplicationNotFound,

    #[error("cannot have more than {0} applications")]
    TooManyApplications(usize),

    #[error("cannot have more than {0} redirect URL for an application")]
    TooManyRedirectUrls(usize),

    #[error("duplicate redirect URL")]
    DuplicateRedirectUrl,

    #[error("invalid redirect URL")]
    InvalidRedirectUrl,

    #[error("invalid authorization code")]
    InvalidAuthorizationCode,
}
