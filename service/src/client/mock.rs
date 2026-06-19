use super::MailClient;
use crate::{client::CaptchaClient, id::AccountId, util::mails};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::debug;

/// Mock mailbox.
#[cfg(feature = "mock")]
pub type Mails = HashMap<AccountId, Vec<mails::Template>>;

/// Mock mail client implementation.
#[derive(Default)]
pub struct MockMailClientImpl {
    mails: Arc<Mutex<Mails>>,
}

impl MockMailClientImpl {
    /// Creates a new mock mail client.
    pub fn boxed_new() -> (Arc<Mutex<Mails>>, Box<dyn MailClient>) {
        let mail_client = Self::default();

        (mail_client.get_mails(), Box::new(mail_client))
    }

    /// Get mail entries.
    pub fn get_mails(&self) -> Arc<Mutex<Mails>> {
        Arc::clone(&self.mails)
    }
}

#[async_trait]
impl MailClient for MockMailClientImpl {
    #[tracing::instrument(skip_all)]
    async fn send(&self, _email: String, id: AccountId, template: mails::Template) {
        let mut debug = self.mails.lock().await;

        let subject = template.subject();
        let _body = template.body("http://example.com/");

        debug!(%id, subject, ?template, "email to account");

        debug.entry(id).or_default().push(template);
    }
}

/// Mock CAPTCHA client implementation.
pub struct MockCaptchaClientImpl;

impl MockCaptchaClientImpl {
    /// Creates a new mock CAPTCHA client, which accepts any request.
    pub fn boxed_new() -> Box<dyn CaptchaClient> {
        Box::new(Self)
    }
}

#[async_trait]
impl CaptchaClient for MockCaptchaClientImpl {
    async fn validate(&self, _id: String) -> bool {
        true
    }
}
