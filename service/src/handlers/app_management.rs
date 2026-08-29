use validator::Validate;

use super::HandlerResult;
use crate::{
    dto,
    handlers::HandlerError,
    id::{AccountId, AppId},
    state::AppState,
};

/// Application management handlers.
///
/// This handlers require session authentication, and on top of that ownership
/// of the application *should be* checked before calling the handler.
pub struct AppManagementHandler(pub AppState);

impl AppManagementHandler {
    /// Checks the application ownership.
    ///
    /// *This handler is intended for middleware.*
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn auth_app_ownership(
        &self,
        account_id: AccountId,
        app_id: AppId,
    ) -> HandlerResult<()> {
        if !self
            .0
            .app_service
            .is_app_owned_by(account_id, app_id)
            .await?
        {
            Err(HandlerError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Changes the application client secret randomly.
    pub async fn roll_client_secret(
        &self,
        app_id: AppId,
    ) -> HandlerResult<dto::response::AppClientSecret> {
        let new_client_secret = self.0.app_service.roll_client_secret(app_id).await?;

        Ok(dto::response::AppClientSecret {
            client_secret: new_client_secret,
        })
    }

    /// Renames the application.
    pub async fn rename_app(
        &self,
        app_id: AppId,
        app_name_dto: dto::request::AppName,
    ) -> HandlerResult<()> {
        app_name_dto.validate()?;

        self.0
            .app_service
            .rename_app(app_id, &app_name_dto.name)
            .await?;

        Ok(())
    }

    /// Deletes the application.
    pub async fn delete_app(&self, app_id: AppId) -> HandlerResult<()> {
        self.0.app_service.delete_app(app_id).await?;

        Ok(())
    }

    /// Adds a new redirect URL to the application.
    pub async fn add_redirect_url(
        &self,
        app_id: AppId,
        redirect_url_dto: dto::request::RedirectUrl,
    ) -> HandlerResult<()> {
        redirect_url_dto.validate()?;

        self.0
            .app_service
            .add_redirect_url(app_id, &redirect_url_dto.redirect_url)
            .await?;

        Ok(())
    }

    /// Removes the redirect URL from the application.
    pub async fn remove_redirect_url(
        &self,
        app_id: AppId,
        redirect_url_dto: dto::request::RedirectUrl,
    ) -> HandlerResult<()> {
        redirect_url_dto.validate()?;

        self.0
            .app_service
            .remove_redirect_url(app_id, &redirect_url_dto.redirect_url)
            .await?;

        Ok(())
    }

    /// Gets redirect URLs registered to the application.
    pub async fn get_redirect_urls(&self, app_id: AppId) -> HandlerResult<Vec<String>> {
        Ok(self.0.app_service.get_redirect_urls(app_id).await?)
    }
}
