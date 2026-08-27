use super::{HandlerError, HandlerResult};
use crate::{dto, state::AppState, token::TokenScope};
use std::net::IpAddr;
use tracing::trace;

/// Account handlers that **does require** escalated privileges.
pub struct AuthorizationHandler(pub AppState);

impl AuthorizationHandler {
    /// Handle privileged tokens.
    ///
    /// See [`TokenScope`].
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(
            account_id = tracing::field::Empty,
            token_scope = tracing::field::Empty,
        )
    )]
    pub async fn auth(self, token_dto: dto::request::Token, ip: IpAddr) -> HandlerResult<()> {
        let base64_encoded_token = token_dto.token;

        let decoded_token = self.0.token_service.decode(&base64_encoded_token)?;

        tracing::Span::current().record("account_id", decoded_token.sub.to_string());
        tracing::Span::current().record(
            "token_scope",
            decoded_token.scope.scope_name(),
        );

        match &decoded_token.scope {
            TokenScope::Session | TokenScope::EmailVerificationSession => {
                trace!("got session token at authorization endpoint");

                Err(HandlerError::Unauthorized)
            }

            TokenScope::AddEmail { email } => {
                self.0
                    .token_service
                    .check_and_revoke_token(&decoded_token)
                    .await?;

                self.0
                    .email_limiting_service
                    .clear_email_limit(email)
                    .await?;
                self.0
                    .email_limiting_service
                    .refund_ip_quota(&ip, email)
                    .await?;

                self.0
                    .account_service
                    .auth_add_email(decoded_token.sub, email)
                    .await?;

                Ok(())
            }

            TokenScope::ChangePrimaryEmail {
                current_primary_email,
                new_primary_email,
            } => {
                self.0
                    .token_service
                    .check_and_revoke_account_tokens_with_scope(&decoded_token)
                    .await?;

                self.0
                    .account_service
                    .auth_change_primary_email(
                        decoded_token.sub,
                        current_primary_email,
                        new_primary_email,
                    )
                    .await?;

                Ok(())
            }

            TokenScope::CompleteSignup { email } => {
                self.0
                    .token_service
                    .check_and_revoke_account_tokens(&decoded_token)
                    .await?;

                self.0
                    .email_limiting_service
                    .clear_email_limit(email)
                    .await?;
                self.0
                    .email_limiting_service
                    .refund_ip_quota(&ip, email)
                    .await?;

                self.0
                    .account_service
                    .auth_complete_signup(decoded_token.sub, email)
                    .await?;

                Ok(())
            }
        }
    }
}
