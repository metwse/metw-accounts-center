use async_trait::async_trait;
use service::{client::MailClient, id::AccountId, util::mails};

/// Mail client for sending emails.
pub struct MailClientImpl;

#[async_trait]
impl MailClient for MailClientImpl {
    async fn send(&self, _email: String, _id: AccountId, _template: mails::Template) {
        todo!()
    }
}
