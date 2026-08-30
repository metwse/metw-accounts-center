use validator::Validate;

use super::HandlerResult;
use crate::{
    dto,
    handlers::HandlerError,
    id::{AccountId, ApplicationId},
    state::AppState,
};

/// Application management handlers.
///
/// This handlers require session authentication, and on top of that ownership
/// of the application *should be* checked before calling the handler.
pub struct ApplicationManagementHandler(pub AppState);

impl ApplicationManagementHandler {
    /// Checks the application ownership.
    ///
    /// *This handler is intended for middleware.*
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn authorize_ownership(
        &self,
        application_id: ApplicationId,
        account_id: AccountId,
    ) -> HandlerResult<()> {
        if !self
            .0
            .application_service
            .is_owned_by(application_id, account_id)
            .await?
        {
            Err(HandlerError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Changes the application client secret randomly.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rotate_client_secret(
        &self,
        application_id: ApplicationId,
    ) -> HandlerResult<dto::response::RotatedClientSecret> {
        let new_client_secret = self
            .0
            .application_service
            .rotate_client_secret(application_id)
            .await?;

        Ok(dto::response::RotatedClientSecret {
            client_secret: new_client_secret,
        })
    }

    /// Renames the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rename(
        &self,
        application_id: ApplicationId,
        app_name_dto: dto::request::ApplicationName,
    ) -> HandlerResult<()> {
        app_name_dto.validate()?;

        self.0
            .application_service
            .rename(application_id, &app_name_dto.name)
            .await?;

        Ok(())
    }

    /// Deletes the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn delete(&self, application_id: ApplicationId) -> HandlerResult<()> {
        self.0.application_service.delete(application_id).await?;

        Ok(())
    }

    /// Adds a new redirect URL to the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_redirect_url(
        &self,
        application_id: ApplicationId,
        redirect_url_dto: dto::request::ApplicationRedirectUrl,
    ) -> HandlerResult<()> {
        redirect_url_dto.validate()?;

        self.0
            .application_service
            .add_redirect_url(application_id, &redirect_url_dto.redirect_url)
            .await?;

        Ok(())
    }

    /// Removes the redirect URL from the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn remove_redirect_url(
        &self,
        application_id: ApplicationId,
        redirect_url_dto: dto::request::ApplicationRedirectUrl,
    ) -> HandlerResult<()> {
        redirect_url_dto.validate()?;

        self.0
            .application_service
            .remove_redirect_url(application_id, &redirect_url_dto.redirect_url)
            .await?;

        Ok(())
    }

    /// Gets redirect URLs registered to the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn list_redirect_urls(
        &self,
        application_id: ApplicationId,
    ) -> HandlerResult<Vec<String>> {
        Ok(self
            .0
            .application_service
            .list_redirect_urls(application_id)
            .await?)
    }
}
