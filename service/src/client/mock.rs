use super::EmailClient;
use crate::{client::CaptchaClient, id::AccountId, util::emails};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::debug;

/// Mock mailbox.
#[cfg(feature = "mock")]
pub type Emails = HashMap<AccountId, Vec<emails::Template>>;

/// Mock email client implementation.
#[derive(Default)]
pub struct MockEmailClientImpl {
    emails: Arc<Mutex<Emails>>,
}

impl MockEmailClientImpl {
    /// Creates a new mock email client.
    pub fn boxed_new() -> Box<Self> {
        let email_client = Self::default();

        Box::new(email_client)
    }

    /// Get email entries.
    pub fn get_emails(&self) -> Arc<Mutex<Emails>> {
        Arc::clone(&self.emails)
    }
}

#[async_trait]
impl EmailClient for MockEmailClientImpl {
    #[tracing::instrument(skip_all)]
    async fn send(&self, _email: String, id: AccountId, template: emails::Template) {
        let mut debug = self.emails.lock().await;

        let subject = template.subject();
        let _body_html = template.body_html("http://example.com/token?=");
        let _body_text = template.body_text("http://example.com/token?=");

        debug!(%id, subject, ?template, "email to account");

        debug.entry(id).or_default().push(template);
    }
}

/// Mock CAPTCHA client implementation.
pub struct MockCaptchaClientImpl;

impl MockCaptchaClientImpl {
    /// Creates a new mock CAPTCHA client, which accepts any request.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self)
    }
}

#[async_trait]
impl CaptchaClient for MockCaptchaClientImpl {
    async fn validate(&self, _id: String) -> bool {
        true
    }
}
