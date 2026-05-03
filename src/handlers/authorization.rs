use crate::{
    client::MailClient,
    service::{AccountService, TokenService},
};
use std::sync::Arc;

/// Account handlers that **does require** escalated privileges.
pub struct AuthorizationHandler {
    account_service: Arc<AccountService>,
    token_service: Arc<TokenService>,
    email_client: Arc<dyn MailClient>,
}

impl AuthorizationHandler {
    /// Creates a new authentication hander.
    pub fn new(
        account_service: Arc<AccountService>,
        token_service: Arc<TokenService>,
        email_client: Arc<dyn MailClient>,
    ) -> Self {
        Self {
            account_service,
            token_service,
            email_client,
        }
    }
}
