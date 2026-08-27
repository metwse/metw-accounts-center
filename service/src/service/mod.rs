mod account;
mod app;
mod email_limiting;
mod token;

mod error;

pub use account::AccountService;
pub use app::AppService;
pub use email_limiting::EmailLimitingService;
pub use token::TokenService;

pub use error::ServiceError;

/// Service result type.
pub type ServiceResult<T> = Result<T, ServiceError>;
