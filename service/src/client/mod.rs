use crate::{id::AccountId, util::emails};
use async_trait::async_trait;

/// Mock client implementations.
#[cfg(feature = "mock")]
pub mod mock;

/// Send emails.
#[async_trait]
pub trait EmailClient: Send + Sync {
    /// Send emails.
    async fn send(&self, email: String, id: AccountId, template: emails::Template);
}

/// Validate CAPTCHAs.
#[async_trait]
pub trait CaptchaClient: Send + Sync {
    /// Validate CAPTCHAs.
    ///
    /// Returns true if validation success.
    async fn validate(&self, id: String) -> bool;
}
