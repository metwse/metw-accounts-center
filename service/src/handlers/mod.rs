mod authentication;
mod authorization;
mod personal;

mod error;

#[cfg(test)]
mod tests;

pub use authentication::AuthenticationHandler;
pub use authorization::AuthorizationHandler;
pub use personal::PersonalHandler;

pub use error::HandlerError;

/// Handler result type.
pub type HandlerResult<T> = Result<T, HandlerError>;
