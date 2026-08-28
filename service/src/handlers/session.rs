use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::{AccountId, AppId},
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
/// [`TokenScope::Session`], `account_id` parameters in methods of this struct
/// extracted from that token.
pub struct SessionHandler(pub AppState);

impl SessionHandler {
    /// Returns account details.
    #[tracing::instrument(
        level = "debug",
        skip(self),
        fields(username = tracing::field::Empty)
    )]
    pub async fn me(self, account_id: AccountId) -> HandlerResult<dto::response::Account> {
        let me = self.0.account_service.me(account_id).await?;

        if let Some(ref username) = me.username {
            tracing::Span::current().record("username", username);
        }

        Ok(me)
    }

    /// Sends [`ConfirmNewEmail`] to add requested email.
    ///
    /// [`ConfirmNewEmail`]: emails::Template::ConfirmNewEmail
    #[tracing::instrument(level = "debug", skip(self, captcha))]
    pub async fn add_email(
        self,
        account_id: AccountId,
        email_dto: dto::request::Email,
        ip: IpAddr,
        captcha: dto::request::Captcha,
    ) -> HandlerResult<()> {
        email_dto.validate()?;
        if !self.0.captcha_client.validate(captcha.captcha).await {
            return Err(HandlerError::InvalidCaptcha);
        }

        let new_email = email_dto.email;

        let (is_email_taken_res, username_res) = tokio::join!(
            self.0.account_service.is_email_taken(&new_email),
            self.0.account_service.get_primary_username(account_id)
        );

        if is_email_taken_res? {
            return Err(HandlerError::Service(ServiceError::EmailTaken));
        }

        let Some(username) = username_res? else {
            return Err(HandlerError::UnexpectedError("account with no username"))?;
        };

        self.0
            .email_limiting_service
            .check_and_consume_quota(&ip, &new_email)
            .await?;

        let add_email_jwt = self.0.token_service.sign(&Token {
            sub: account_id,
            scope: TokenScope::AddEmail {
                email: new_email.clone(),
            },
        });

        let template = emails::Template::ConfirmNewEmail {
            username,
            email: new_email.clone(),
            token: add_email_jwt,
        };

        self.0
            .email_client
            .send(new_email, account_id, template)
            .await;

        Ok(())
    }

    /// Removes the email if it is not account's primary email.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn delete_email(
        self,
        account_id: AccountId,
        email_dto: dto::request::Email,
    ) -> HandlerResult<()> {
        email_dto.validate()?;

        let email = email_dto.email;

        self.0
            .account_service
            .remove_email_if_not_primary(account_id, &email)
            .await?;

        Ok(())
    }

    /// Sends [`ConfirmPrimaryEmailChange`] email to current primary email.
    ///
    /// [`ConfirmPrimaryEmailChange`]: emails::Template::ConfirmPrimaryEmailChange
    #[tracing::instrument(level = "debug", skip(self, captcha))]
    pub async fn set_primary_email(
        self,
        account_id: AccountId,
        email_dto: dto::request::Email,
        captcha: dto::request::Captcha,
    ) -> HandlerResult<()> {
        email_dto.validate()?;
        if !self.0.captcha_client.validate(captcha.captcha).await {
            return Err(HandlerError::InvalidCaptcha);
        }

        let new_primary_email = email_dto.email;

        let (current_primary_email_res, username_res) = tokio::join!(
            self.0.account_service.get_primary_email(account_id),
            self.0.account_service.get_primary_username(account_id)
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
            return Err(HandlerError::AlreadyPrimaryEmail);
        }

        if !self
            .0
            .account_service
            .is_email_taken_by(account_id, &new_primary_email)
            .await?
        {
            return Err(HandlerError::Service(ServiceError::EmailNotFound));
        };

        let change_primary_email_jwt = self.0.token_service.sign(&Token {
            sub: account_id,
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
            .send(current_primary_email, account_id, template)
            .await;

        Ok(())
    }

    /// Update account password.
    #[tracing::instrument(level = "debug", skip(self, change_password_dto))]
    pub async fn change_password(
        &self,
        account_id: AccountId,
        change_password_dto: dto::request::ChangePassword,
    ) -> HandlerResult<()> {
        change_password_dto.validate()?;

        self.0
            .account_service
            .change_password(account_id, &change_password_dto)
            .await?;

        Ok(())
    }

    /// Gets list of the applications owned by the account.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_my_apps(
        &self,
        account_id: AccountId,
    ) -> HandlerResult<Vec<dto::response::BasicAppInfo>> {
        let apps = self.0.app_service.get_apps(account_id).await?;

        Ok(apps
            .into_iter()
            .map(|app| dto::response::BasicAppInfo {
                app_id: app.app_id,
                name: app.name,
            })
            .collect())
    }

    /// Register a new application.
    #[tracing::instrument(level = "debug", skip(self, captcha))]
    pub async fn create_app(
        &self,
        account_id: AccountId,
        create_app_dto: dto::request::CreateApp,
        captcha: dto::request::Captcha,
    ) -> HandlerResult<dto::response::AppInfo> {
        create_app_dto.validate()?;
        if !self.0.captcha_client.validate(captcha.captcha).await {
            return Err(HandlerError::InvalidCaptcha);
        }

        let app = self
            .0
            .app_service
            .create_app(account_id, &create_app_dto.name)
            .await?;

        Ok(dto::response::AppInfo {
            app_id: app.app_id,
            name: create_app_dto.name,
            client_secret: app.client_secret,
        })
    }
}
