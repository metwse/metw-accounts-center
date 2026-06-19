use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::AccountId,
    service::ServiceError,
    state::AppState,
    token::{Token, TokenScope},
    util::mails,
};
use validator::Validate;

/// Account handlers that does not require escalated privileges.
///
/// This handlers *should be* protected using tokens with
/// [`TokenScope::Session`], `id` parameters in methods of this struct
/// extracted from that token.
pub struct SessionHandler(pub AppState);

impl SessionHandler {
    /// Returns account details.
    #[tracing::instrument(skip(self))]
    pub async fn me(self, id: AccountId) -> HandlerResult<dto::response::Account> {
        Ok(self.0.account_service.me(id).await?)
    }

    /// Sends [`ConfirmNewEmail`] to add requested email.
    ///
    /// [`ConfirmNewEmail`]: mails::Template::ConfirmNewEmail
    #[tracing::instrument(skip(self))]
    pub async fn add_email(self, id: AccountId, email: dto::request::Email) -> HandlerResult<()> {
        email.validate()?;

        let email = email.email;

        if self.0.account_service.is_email_taken(&email).await? {
            return Err(ServiceError::EmailTaken)?;
        }

        let Some(username) = self.0.account_service.get_primary_username(id).await? else {
            return Err(HandlerError::UnexpectedError("account with no username"))?;
        };

        let add_email_jwt = self.0.token_service.sign(&Token::new(
            id,
            TokenScope::AddEmail {
                email: email.clone(),
            },
        ));

        let template = mails::Template::ConfirmNewEmail {
            username,
            email: email.clone(),
            token: add_email_jwt,
        };

        self.0.mail_client.send(email, id, template).await;

        Ok(())
    }

    /// Removes the email if it is not account's primary email.
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

    /// Sends [`ConfirmPrimaryEmailChange`] email to current primary email.
    ///
    /// [`ConfirmPrimaryEmailChange`]: mails::Template::ConfirmPrimaryEmailChange
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

        let Some(username) = self.0.account_service.get_primary_username(id).await? else {
            return Err(HandlerError::UnexpectedError("account with no username"))?;
        };

        if !self
            .0
            .account_service
            .is_email_taken_by(id, &new_primary_email)
            .await?
        {
            return Err(ServiceError::EmailNotFound)?;
        };

        let change_primary_email_jwt = self.0.token_service.sign(&Token::new(
            id,
            TokenScope::ChangePrimaryEmail {
                current_primary_email: current_primary_email.clone(),
                new_primary_email: new_primary_email.clone(),
            },
        ));

        let template = mails::Template::ConfirmPrimaryEmailChange {
            username,
            current_primary_email: current_primary_email.clone(),
            new_primary_email,
            token: change_primary_email_jwt,
        };

        self.0
            .mail_client
            .send(current_primary_email, id, template)
            .await;

        Ok(())
    }
}
