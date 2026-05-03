use super::super::MailClient;
use crate::{entity, util::templated_mails};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub(crate) type Mails = HashMap<entity::AccountId, Vec<templated_mails::Template>>;

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
    async fn send(&self, id: entity::AccountId, template: templated_mails::Template) {
        let mut debug = self.mails.lock().await;

        let subject = template.subject();
        let body = template.body();

        // TODO: use trancing instead
        println!("--- EMAIL TO: {id} ---\nSub: {subject}\n\n{body}\n----------------------");

        debug.entry(id).or_default().push(template);
    }
}
