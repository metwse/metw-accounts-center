use crate::{
    client::MailClient,
    dto, entity,
    handlers::HandlerResult,
    service::{AccountService, TokenService},
};
use std::sync::Arc;

/// Account handlers that does not require escalated privileges.
///
/// This handlers *should be* protected using tokens with
/// [`TokenScope::Authenticate`], `id` parameters in methods of this struct
/// extracted from that token.
///
/// [`TokenScope::Authenticate`]: crate::token::TokenScope::Authenticate
pub struct AccountHandler {
    account_service: Arc<AccountService>,
    _token_service: Arc<TokenService>,
    _email_client: Arc<dyn MailClient>,
}

impl AccountHandler {
    /// Creates a new account hander.
    pub fn new(
        account_service: Arc<AccountService>,
        _token_service: Arc<TokenService>,
        _email_client: Arc<dyn MailClient>,
    ) -> Self {
        Self {
            account_service,
            _token_service,
            _email_client,
        }
    }

    /// GET `/me`
    pub async fn me(&self, id: entity::AccountId) -> HandlerResult<dto::response::Account> {
        Ok(self.account_service.me(id).await?)
    }
}
