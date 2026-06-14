use crate::{
    client::MailClient,
    service::{AccountService, TokenService},
};
use std::sync::Arc;

#[cfg(test)]
use crate::{
    client::mock::{MockMailClientImpl, Mails},
    repo::mock::{MockAccountRepoImpl, MockTokenRepoImpl},
};
#[cfg(test)]
use tokio::sync::Mutex;

/// Application-wide state.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct State {
    pub account_service: Arc<AccountService>,
    pub token_service: Arc<TokenService>,
    pub mail_client: Arc<dyn MailClient>,
    pub email_callback_url: Arc<String>,
    #[cfg(test)]
    pub emails: Arc<Mutex<Mails>>,
}

impl State {
    /// Creates a new mock state.
    #[cfg(any(test, doc))]
    pub fn new_mock() -> Self {
        let account_service = AccountService::new(MockAccountRepoImpl::boxed_new());
        let token_service =
            TokenService::new(MockTokenRepoImpl::boxed_new(), b"secret123".to_vec());
        let (emails, mail_client) = MockMailClientImpl::boxed_new();

        Self {
            account_service: account_service.into(),
            token_service: token_service.into(),
            mail_client: mail_client.into(),
            email_callback_url: Arc::new("http://example.com".to_string()),
            emails,
        }
    }
}
