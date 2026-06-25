use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::AccountId,
    service::ServiceError,
    state::AppState,
    token::{Token, TokenScope},
    util::emails,
};
use std::net::IpAddr;
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
    /// [`ConfirmNewEmail`]: emails::Template::ConfirmNewEmail
    #[tracing::instrument(skip(self))]
    pub async fn add_email(
        self,
        id: AccountId,
        email_dto: dto::request::Email,
        ip: IpAddr,
    ) -> HandlerResult<()> {
        email_dto.validate()?;

        let new_email = email_dto.email;

        let (is_email_taken_res, username_res) = tokio::join!(
            self.0.account_service.is_email_taken(&new_email),
            self.0.account_service.get_primary_username(id)
        );

        if is_email_taken_res? {
            return Err(ServiceError::EmailTaken)?;
        }

        let Some(username) = username_res? else {
            return Err(HandlerError::UnexpectedError("account with no username"))?;
        };

        self.0
            .email_limiting_service
            .check_and_consume_quota(&ip, &new_email)
            .await?;

        let add_email_jwt = self.0.token_service.sign(&Token {
            id,
            scope: TokenScope::AddEmail {
                email: new_email.clone(),
            },
        });

        let template = emails::Template::ConfirmNewEmail {
            username,
            email: new_email.clone(),
            token: add_email_jwt,
        };

        self.0.email_client.send(new_email, id, template).await;

        Ok(())
    }

    /// Removes the email if it is not account's primary email.
    #[tracing::instrument(skip(self))]
    pub async fn delete_email(
        self,
        id: AccountId,
        email_dto: dto::request::Email,
    ) -> HandlerResult<()> {
        email_dto.validate()?;

        let email = email_dto.email;

        self.0
            .account_service
            .remove_email_if_not_primary(id, &email)
            .await?;

        Ok(())
    }

    /// Sends [`ConfirmPrimaryEmailChange`] email to current primary email.
    ///
    /// [`ConfirmPrimaryEmailChange`]: emails::Template::ConfirmPrimaryEmailChange
    #[tracing::instrument(skip(self))]
    pub async fn set_primary_email(
        self,
        id: AccountId,
        email_dto: dto::request::Email,
    ) -> HandlerResult<()> {
        email_dto.validate()?;

        let new_primary_email = email_dto.email;

        let (current_primary_email_res, username_res) = tokio::join!(
            self.0.account_service.get_primary_email(id),
            self.0.account_service.get_primary_username(id)
        );

        let Some(current_primary_email) = current_primary_email_res? else {
            return Err(HandlerError::UnexpectedError(
                "account with no primary email",
            ))?;
        };

        let Some(username) = username_res? else {
            return Err(HandlerError::UnexpectedError("account with no username"))?;
        };

        if current_primary_email == new_primary_email {
            return Err(HandlerError::AlreadyPrimaryEmail)?;
        }

        if !self
            .0
            .account_service
            .is_email_taken_by(id, &new_primary_email)
            .await?
        {
            return Err(ServiceError::EmailNotFound)?;
        };

        let change_primary_email_jwt = self.0.token_service.sign(&Token {
            id,
            scope: TokenScope::ChangePrimaryEmail {
                current_primary_email: current_primary_email.clone(),
                new_primary_email: new_primary_email.clone(),
            },
        });

        let template = emails::Template::ConfirmPrimaryEmailChange {
            username,
            current_primary_email: current_primary_email.clone(),
            new_primary_email,
            token: change_primary_email_jwt,
        };

        self.0
            .email_client
            .send(current_primary_email, id, template)
            .await;

        Ok(())
    }
}
