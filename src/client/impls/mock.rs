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
    pub fn shared_new_with_emails() -> (Arc<Mutex<Mails>>, Arc<dyn MailClient>) {
        let res = Self::default();

        (Arc::clone(&res.mails), Arc::new(res))
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
