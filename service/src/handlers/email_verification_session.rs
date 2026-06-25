use super::HandlerResult;
use crate::{
    dto,
    handlers::HandlerError,
    id::AccountId,
    service::ServiceError,
    state::AppState,
    token::{Token, TokenScope},
    util::emails,
};
use std::net::IpAddr;
use validator::Validate;

/// Account handlers before the email verification.
///
/// This handlers *should be* protected using tokens with
/// [`TokenScope::EmailVerificationSession`]. See [`SessionHandler`] for
/// details.
///
/// [`SessionHandler`]: super::SessionHandler
pub struct EmailVerificationSessionHandler(pub AppState);

impl EmailVerificationSessionHandler {
    /// Resends the sign up email.
    ///
    /// Sends a [`ConfirmSignup`] email.
    ///
    /// [`ConfirmSignup`]: emails::Template::ConfirmSignup
    #[tracing::instrument(skip(self))]
    pub async fn retry_signup(
        self,
        id: AccountId,
        email_dto: dto::request::Email,
        ip: IpAddr,
        captcha: dto::request::Captcha,
    ) -> HandlerResult<()> {
        email_dto.validate()?;
        if !self.0.captcha_client.validate(captcha.captcha).await {
            return Err(HandlerError::InvalidCaptcha);
        }

        let email = email_dto.email;

        if self.0.account_service.is_email_taken(&email).await? {
            return Err(ServiceError::EmailTaken)?;
        }

        let Some(username) = self.0.account_service.get_primary_username(id).await? else {
            return Err(HandlerError::UnexpectedError("account with no username"));
        };

        self.0
            .email_limiting_service
            .check_and_consume_quota(&ip, &email)
            .await?;

        let complete_signup_jwt = self.0.token_service.sign(&Token {
            id,
            scope: TokenScope::CompleteSignup {
                email: email.clone(),
            },
        });

        let template = emails::Template::ConfirmSignup {
            username,
            token: complete_signup_jwt,
        };

        self.0.email_client.send(email, id, template).await;

        Ok(())
    }
}
