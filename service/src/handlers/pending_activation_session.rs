use super::HandlerResult;
use crate::{
    dto,
    handlers::HandlerError,
    id::AccountId,
    service::ServiceError,
    state::State,
    token::{Token, TokenScope},
    util::mails,
};
use validator::Validate;

/// Account handlers before the email verification.
///
/// This handlers *should be* protected using tokens with
/// [`TokenScope::PendingActivationSession`]. See [`SessionHandler`] for
/// details.
///
/// [`SessionHandler`]: super::SessionHandler
pub struct PendingActivationSessionHandler(pub State);

impl PendingActivationSessionHandler {
    /// Resends the sign up email.
    ///
    /// Sends a [`ConfirmSignup`] email.
    ///
    /// [`ConfirmSignup`]: mails::Template::ConfirmSignup
    #[tracing::instrument(skip(self))]
    pub async fn retry_signup(
        self,
        id: AccountId,
        email: dto::request::Email,
    ) -> HandlerResult<()> {
        email.validate()?;

        let email = email.email;

        if self.0.account_service.is_email_taken(&email).await? {
            return Err(ServiceError::EmailTaken)?;
        }

        let Some(username) = self.0.account_service.get_primary_username(id).await? else {
            return Err(HandlerError::UnexpectedError("account with no username"));
        };

        let complete_signup_jwt = self.0.token_service.sign(&Token::new(
            id,
            TokenScope::CompleteSignup {
                email: email.clone(),
            },
        ));

        let template = mails::Template::ConfirmSignup {
            username,
            token: complete_signup_jwt,
        };

        self.0.mail_client.send(email, id, template).await;

        Ok(())
    }
}
