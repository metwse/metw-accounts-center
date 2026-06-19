mod authentication;
mod authorization;
mod pending_activation_session;
mod session;

mod error;

pub use authentication::AuthenticationHandler;
pub use authorization::AuthorizationHandler;
pub use pending_activation_session::PendingActivationSessionHandler;
pub use session::SessionHandler;

pub use error::HandlerError;

/// Handler result type.
pub type HandlerResult<T> = Result<T, HandlerError>;
