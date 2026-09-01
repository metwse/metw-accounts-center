/// Error types for the metw account center API.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid client secret or unauthorized account access.
    #[error("Invalid client secret or unauthorized account access.")]
    Unauthorized,

    /// Provided authorization code is not valid.
    #[error("Provided authorization code is not valid.")]
    InvalidAuthorizationCode,

    /// API returned an unexpected error.
    #[error("API returned an unexpected error.")]
    UnknownError,

    /// The error originated from reqwest.
    #[error("The error originated from reqwest.")]
    ReqwestError(#[from] reqwest::Error),
}
