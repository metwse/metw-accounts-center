use crate::{id::AccountId, util::templated_mails};
use async_trait::async_trait;

/// Mock client implementations.
#[cfg(feature = "mock")]
pub mod mock;

/// Send emails.
#[async_trait]
pub trait MailClient {
    /// Send emails.
    async fn send(&self, email: String, id: AccountId, template: templated_mails::Template);
}

/// Validate CAPTCHAs.
#[async_trait]
pub trait CaptchaClient {
    /// Validate CAPTCHAs.
    async fn validate(&self, id: String) -> bool;
}
