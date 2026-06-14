mod account;
mod token;

mod error;

#[cfg(test)]
mod tests;

pub use account::AccountService;
pub use token::TokenService;

pub use error::ServiceError;

/// Service result type.
pub type ServiceResult<T> = Result<T, ServiceError>;
