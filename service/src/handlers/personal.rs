use validator::Validate;

use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::AccountId,
    service::ServiceError,
    state::State,
    token::{Token, TokenScope},
    util::mails,
};
use std::time::Duration;

static ADD_EMAIL_TOKEN_VALID_FOR: Duration = Duration::from_secs(60 * 60);
static SET_PRIMARY_MAIL_VALID_FOR: Duration = Duration::from_secs(60 * 10);

/// Account handlers that does not require escalated privileges.
///
/// This handlers *should be* protected using tokens with
/// [`TokenScope::Authenticate`], `id` parameters in methods of this struct
/// extracted from that token.
///
/// [`TokenScope::Authenticate`]: crate::token::TokenScope::Authenticate
pub struct PersonalHandler(pub State);

impl PersonalHandler {
    /// GET `/me`
    #[tracing::instrument(skip(self))]
    pub async fn me(self, id: AccountId) -> HandlerResult<dto::response::Account> {
        Ok(self.0.account_service.me(id).await?)
    }

    /// POST `/me/emails`
    ///
    /// Add a new email to the account. Sends verification code to requested
    /// email.
    #[tracing::instrument(skip(self))]
    pub async fn add_email(self, id: AccountId, email: dto::request::Email) -> HandlerResult<()> {
        email.validate()?;

        let email = email.email;

        if self.0.account_service.is_email_taken(&email).await? {
            return Err(ServiceError::EmailTaken)?;
        }

        let add_email_jwt = self.0.token_service.sign(&Token::new(
            id,
            TokenScope::AddEmail(email.clone()),
            ADD_EMAIL_TOKEN_VALID_FOR,
        ));

        let template = mails::Template::AddEmail {
            email: email.clone(),
            add_email_jwt,
        };

        self.0.mail_client.send(email, id, template).await;

        Ok(())
    }

    /// DELETE `/me/emails/<email>`
    ///
    /// Remove the email if it is not primary email.
    #[tracing::instrument(skip(self))]
    pub async fn delete_email(
        self,
        id: AccountId,
        email: dto::request::Email,
    ) -> HandlerResult<()> {
        email.validate()?;

        let email = email.email;

        self.0
            .account_service
            .remove_email_if_not_primary(id, &email)
            .await?;

        Ok(())
    }

    /// POST `/me/emails/<email>/set-primary`
    ///
    /// Set the email as account's primary mail.
    #[tracing::instrument(skip(self))]
    pub async fn set_primary_mail(
        self,
        id: AccountId,
        new_primary_email: dto::request::Email,
    ) -> HandlerResult<()> {
        new_primary_email.validate()?;

        let new_primary_email = new_primary_email.email;

        let Some(current_primary_email) = self.0.account_service.get_primary_email(id).await?
        else {
            return Err(HandlerError::UnexpectedError("account with no email"));
        };

        if !self
            .0
            .account_service
            .is_email_taken_by(id, &new_primary_email)
            .await?
        {
            return Err(ServiceError::EmailNotFound)?;
        };

        let set_primary_mail_jwt = self.0.token_service.sign(&Token::new(
            id,
            TokenScope::SetPrimaryEmail {
                current_primary_email: current_primary_email.clone(),
                new_primary_email: new_primary_email.clone(),
            },
            SET_PRIMARY_MAIL_VALID_FOR,
        ));

        let template = mails::Template::SetPrimaryEmail {
            current_primary_email: current_primary_email.clone(),
            new_primary_email,
            set_primary_mail_jwt,
        };

        self.0
            .mail_client
            .send(current_primary_email, id, template)
            .await;

        Ok(())
    }
}
