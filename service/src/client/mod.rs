use crate::{id::AccountId, util::mails};
use async_trait::async_trait;

/// Mock client implementations.
#[cfg(feature = "mock")]
pub mod mock;

/// Send emails.
#[async_trait]
pub trait MailClient: Send + Sync {
    /// Send emails.
    async fn send(&self, email: String, id: AccountId, template: mails::Template);
}

/// Validate CAPTCHAs.
#[async_trait]
pub trait CaptchaClient: Send + Sync {
    /// Validate CAPTCHAs.
    async fn validate(&self, id: String) -> bool;
}
