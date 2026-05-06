use tracing::trace;

use super::{HandlerError, HandlerResult};
use crate::{
    service::{AccountService, TokenService},
    token::TokenScope,
};
use std::sync::Arc;

/// Account handlers that **does require** escalated privileges.
pub struct AuthorizationHandler {
    account_service: Arc<AccountService>,
    token_service: Arc<TokenService>,
}

impl AuthorizationHandler {
    /// Creates a new authentication hander.
    pub fn new(account_service: Arc<AccountService>, token_service: Arc<TokenService>) -> Self {
        Self {
            account_service,
            token_service,
        }
    }

    /// Handle privileged tokens.
    #[tracing::instrument(skip_all)]
    pub async fn auth(&self, base64_encoded_token: String) -> HandlerResult<()> {
        let token = self.token_service.revoke(&base64_encoded_token).await?;

        trace!(account_id = %token.id, variant = token.scope.variant_name());

        match token.scope {
            TokenScope::Authenticate => Err(HandlerError::Unauthorized),

            TokenScope::AddEmail(email) => {
                Ok(self.account_service.auth_add_email(token.id, email).await?)
            }

            TokenScope::SetPrimaryEmail {
                current_primary_email,
                new_primary_email,
            } => Ok(self
                .account_service
                .auth_change_primary_email(token.id, current_primary_email, new_primary_email)
                .await?),

            TokenScope::Signup { email } => Ok(self
                .account_service
                .auth_complete_signup(token.id, email)
                .await?),
        }
    }
}
