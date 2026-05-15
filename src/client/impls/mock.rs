use super::super::MailClient;
use crate::{id::AccountId, util::templated_mails};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::debug;

pub(crate) type Mails = HashMap<AccountId, Vec<templated_mails::Template>>;

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
    async fn send(&self, id: AccountId, template: templated_mails::Template) {
        let mut debug = self.mails.lock().await;

        let subject = template.subject();
        let _body = template.body();

        debug!(%id, subject, ?template, "email to account");

        debug.entry(id).or_default().push(template);
    }
}
