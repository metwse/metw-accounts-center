use super::{HandlerError, HandlerResult};
use crate::{state::State, token::TokenScope};
use tracing::trace;

/// Account handlers that **does require** escalated privileges.
pub struct AuthorizationHandler(pub State);

impl AuthorizationHandler {
    /// Handle privileged tokens.
    #[tracing::instrument(skip_all)]
    pub async fn auth(self, base64_encoded_token: String) -> HandlerResult<()> {
        let token = self.0.token_service.revoke(&base64_encoded_token).await?;

        trace!(account_id = %token.id, variant = token.scope.variant_name());

        match token.scope {
            TokenScope::Authenticate => Err(HandlerError::Unauthorized),

            TokenScope::AddEmail(email) => {
                self.0
                    .account_service
                    .auth_add_email(token.id, &email)
                    .await?;

                Ok(())
            }

            TokenScope::SetPrimaryEmail {
                current_primary_email,
                new_primary_email,
            } => {
                self.0
                    .account_service
                    .auth_change_primary_email(token.id, &current_primary_email, &new_primary_email)
                    .await?;

                Ok(())
            }

            TokenScope::Signup { email } => {
                self.0
                    .account_service
                    .auth_complete_signup(token.id, &email)
                    .await?;

                Ok(())
            }
        }
    }
}
