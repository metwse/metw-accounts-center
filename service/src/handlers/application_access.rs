use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::{AccountId, ApplicationId},
    service::ServiceError,
    state::AppState,
};
use validator::Validate;

/// Application access handlers.
///
/// Registered applications access account details and exchange authorization
/// code with this handler.
pub struct ApplicationAccessHandler(pub AppState);

impl ApplicationAccessHandler {
    /// Checks the application client secret.
    ///
    /// *This handler is intended for middleware.*
    #[tracing::instrument(level = "debug", skip(self, client_secret))]
    pub async fn authenticate(
        &self,
        application_id: ApplicationId,
        client_secret: String,
    ) -> HandlerResult<()> {
        if !self
            .0
            .application_service
            .check_client_secret(application_id, &client_secret)
            .await?
        {
            Err(HandlerError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Exchange the authorization code with account ID.
    #[tracing::instrument(level = "debug", skip(self, authorization_code_dto))]
    pub async fn exchange(
        &self,
        application_id: ApplicationId,
        authorization_code_dto: dto::request::AuthorizationCode,
    ) -> HandlerResult<dto::response::AuthorizationCodeExchangeResult> {
        authorization_code_dto.validate()?;
        let authorization_code = authorization_code_dto.authorization_code;

        let account_id = self
            .0
            .authorization_code_service
            .consume(application_id, &authorization_code)
            .await?;

        if !self
            .0
            .application_service
            .check_consent(account_id, application_id)
            .await?
        {
            Err(ServiceError::InvalidAuthorizationCode)?;
        }

        Ok(dto::response::AuthorizationCodeExchangeResult { account_id })
    }

    /// Gets account if the application has been authorized.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_account(
        &self,
        application_id: ApplicationId,
        account_id: AccountId,
    ) -> HandlerResult<dto::response::Account> {
        let account = self.0.account_service.get(account_id).await?;

        if self
            .0
            .application_service
            .check_consent(account_id, application_id)
            .await?
        {
            Ok(account)
        } else {
            Err(HandlerError::Unauthorized)
        }
    }
}
