mod authentication;
mod authorization;
mod email_verification_session;
mod session;

pub use authentication::authentication_routes;
pub use authorization::authorization_routes;
pub use email_verification_session::email_verification_session_routes;
pub use session::session_routes;

// TODO: utoipa docs
// TODO: tests
