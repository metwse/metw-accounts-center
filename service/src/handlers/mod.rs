mod application_access;
mod application_management;
mod authentication;
mod email_verification_session;
mod session;
mod token_action;

mod error;

pub use application_access::ApplicationAccessHandler;
pub use application_management::ApplicationManagementHandler;
pub use authentication::AuthenticationHandler;
pub use email_verification_session::EmailVerificationSessionHandler;
pub use session::SessionHandler;
pub use token_action::TokenActionHandler;

pub use error::HandlerError;

/// Handler result type.
pub type HandlerResult<T> = Result<T, HandlerError>;
