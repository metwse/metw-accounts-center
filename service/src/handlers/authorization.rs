use super::{HandlerError, HandlerResult};
use crate::{dto, state::AppState, token::TokenScope};
use tracing::trace;

/// Account handlers that **does require** escalated privileges.
pub struct AuthorizationHandler(pub AppState);

impl AuthorizationHandler {
    /// Handle privileged tokens.
    ///
    /// See [`TokenScope`].
    #[tracing::instrument(skip_all)]
    pub async fn auth(self, token_dto: dto::request::Token) -> HandlerResult<()> {
        let base64_encoded_token = token_dto.token;

        let token = self.0.token_service.revoke(&base64_encoded_token).await?;

        trace!(account_id = %token.id, variant = token.scope.scope_name());

        match token.scope {
            TokenScope::Session | TokenScope::EmailVerificationSession => {
                Err(HandlerError::Unauthorized)
            }

            TokenScope::AddEmail { email } => {
                self.0
                    .account_service
                    .auth_add_email(token.id, &email)
                    .await?;

                Ok(())
            }

            TokenScope::ChangePrimaryEmail {
                current_primary_email,
                new_primary_email,
            } => {
                self.0
                    .account_service
                    .auth_change_primary_email(token.id, &current_primary_email, &new_primary_email)
                    .await?;

                Ok(())
            }

            TokenScope::CompleteSignup { email } => {
                self.0
                    .account_service
                    .auth_complete_signup(token.id, &email)
                    .await?;

                Ok(())
            }
        }
    }
}
