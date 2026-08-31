use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::{AccountId, ApplicationId},
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
    pub async fn get_current_account(
        self,
        account_id: AccountId,
    ) -> HandlerResult<dto::response::Account> {
        let account = self.0.account_service.get(account_id).await?;

        if let Some(ref username) = account.username {
            tracing::Span::current().record("username", username);
        }

        Ok(account)
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
    pub async fn remove_email(
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
    pub async fn change_primary_email(
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
            .is_email_owned_by(account_id, &new_primary_email)
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
    pub async fn get_owned_applications(
        &self,
        account_id: AccountId,
    ) -> HandlerResult<Vec<dto::response::ApplicationSummary>> {
        let applications = self.0.application_service.list_owned_by(account_id).await?;

        Ok(applications
            .into_iter()
            .map(|application| dto::response::ApplicationSummary {
                application_id: application.application_id,
                name: application.name,
            })
            .collect())
    }

    /// Register a new application.
    #[tracing::instrument(
        level = "debug",
        skip(self, captcha),
        fields(application_id = tracing::field::Empty)
    )]
    pub async fn create_application(
        &self,
        account_id: AccountId,
        create_app_dto: dto::request::ApplicationName,
        captcha: dto::request::Captcha,
    ) -> HandlerResult<dto::response::CreatedApplication> {
        create_app_dto.validate()?;
        if !self.0.captcha_client.validate(captcha.captcha).await {
            return Err(HandlerError::InvalidCaptcha);
        }

        let application = self
            .0
            .application_service
            .create(account_id, &create_app_dto.name)
            .await?;

        tracing::Span::current().record("application_id", application.application_id.to_string());

        Ok(dto::response::CreatedApplication {
            application_id: application.application_id,
            name: create_app_dto.name,
            client_secret: application.client_secret,
        })
    }

    /// Gets list of the applications owned by the account.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_application_consents(
        &self,
        account_id: AccountId,
        pagination_dto: dto::request::ConsentPagination,
    ) -> HandlerResult<Vec<dto::response::ApplicationConsent>> {
        let authorized_applications = self
            .0
            .application_service
            .list_consents(account_id, pagination_dto.after_application_id)
            .await?;

        Ok(authorized_applications
            .into_iter()
            .map(|consent| dto::response::ApplicationConsent {
                application_id: consent.application_id,
                name: consent.name,
                created_at_timestamp: consent.created_at.timestamp().to_string(),
            })
            .collect())
    }

    /// Authorize the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn authorize_application(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
        redirect_url_dto: dto::request::ApplicationRedirectUrl,
    ) -> HandlerResult<()> {
        redirect_url_dto.validate()?;
        let redirect_url = redirect_url_dto.redirect_url;

        if !self
            .0
            .application_service
            .has_redirect_url(application_id, &redirect_url)
            .await?
        {
            Err(ServiceError::InvalidRedirectUrl)?;
        }

        Ok(self
            .0
            .application_service
            .add_consent(account_id, application_id)
            .await?)
    }

    /// Create an authorization code for application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn create_authorization_code(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> HandlerResult<dto::response::AuthorizationCode> {
        if !self
            .0
            .application_service
            .check_consent(account_id, application_id)
            .await?
        {
            return Err(HandlerError::Unauthorized);
        }

        let authorization_code = self
            .0
            .authorization_code_service
            .create(account_id, application_id)
            .await?;

        Ok(dto::response::AuthorizationCode { authorization_code })
    }

    /// Remove authorization of the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn unauthorize_application(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> HandlerResult<()> {
        Ok(self
            .0
            .application_service
            .remove_consent(account_id, application_id)
            .await?)
    }

    /// Check the authorization of the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_application_consent_status(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> HandlerResult<dto::response::ApplicationConsentStatus> {
        Ok(dto::response::ApplicationConsentStatus {
            is_authorized: self
                .0
                .application_service
                .check_consent(account_id, application_id)
                .await?,
        })
    }
}
