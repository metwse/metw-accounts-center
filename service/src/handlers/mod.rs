mod authentication;
mod authorization;
mod email_verification_session;
mod session;

mod error;

pub use authentication::AuthenticationHandler;
pub use authorization::AuthorizationHandler;
pub use email_verification_session::EmailVerificationSessionHandler;
pub use session::SessionHandler;

pub use error::HandlerError;

/// Handler result type.
pub type HandlerResult<T> = Result<T, HandlerError>;
