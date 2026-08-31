use crate::{
    id::{AccountId, ApplicationId},
    repo::AuthorizationCodeRepo,
    service::{ServiceError, ServiceResult},
};

/// Authorization code state.
pub struct AuthorizationCodeService {
    repo: Box<dyn AuthorizationCodeRepo>,
}

impl AuthorizationCodeService {
    /// Creates a new authorization code service.
    pub fn new(repo: Box<dyn AuthorizationCodeRepo>) -> Self {
        Self { repo }
    }

    /// Creates a new authorization code.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn create(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> ServiceResult<String> {
        Ok(self.repo.create(account_id, application_id).await?)
    }

    /// Consumes the authorization code.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn consume(
        &self,
        application_id: ApplicationId,
        authorization_code: &str,
    ) -> ServiceResult<AccountId> {
        self.repo
            .consume(application_id, authorization_code)
            .await?
            .ok_or(ServiceError::InvalidAuthorizationCode)
    }
}
