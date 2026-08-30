use super::{ServiceError, ServiceResult};
use crate::{
    dto, entity,
    id::{AccountId, ApplicationId},
    repo::{ApplicationRepo, limits},
    util::client_secret,
};

/// User-registered app state.
pub struct ApplicationService {
    repo: Box<dyn ApplicationRepo>,
}

impl ApplicationService {
    /// Creates a new account service.
    pub fn new(repo: Box<dyn ApplicationRepo>) -> Self {
        Self { repo }
    }

    /// Register a new application, and return its client secret and ID.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn create(
        &self,
        account_id: AccountId,
        name: &str,
    ) -> ServiceResult<dto::service::CreatedApplication> {
        // Best-effort early fail: Repository should also check this.
        if self.repo.count_by_owner(account_id).await?
            >= limits::application_repo::MAXIMUM_APPLICATION_COUNT
        {
            return Err(ServiceError::TooManyApplications(
                limits::application_repo::MAXIMUM_APPLICATION_COUNT,
            ));
        }

        let application_id = ApplicationId::unique();
        let client_secret = client_secret::random_client_secret();
        let client_secret_hash = client_secret::hash_client_secret(&client_secret);

        let application = entity::Application {
            application_id,
            owner_account_id: account_id,
            name: name.to_string(),
            client_secret_hash,
        };

        let mut transaction = self.repo.begin_transaction().await?;
        transaction.insert(application).await?;
        transaction.commit().await?;

        Ok(dto::service::CreatedApplication {
            application_id,
            client_secret,
        })
    }

    /// Delete the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn delete(&self, application_id: ApplicationId) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction.delete(application_id).await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Get the list of registered applications.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn list_owned_by(
        &self,
        account_id: AccountId,
    ) -> ServiceResult<Vec<dto::repo::OwnedApplicationSummary>> {
        Ok(self.repo.list_by_owner(account_id).await?)
    }

    /// Check app ownership.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn is_owned_by(
        &self,
        application_id: ApplicationId,
        account_id: AccountId,
    ) -> ServiceResult<bool> {
        Ok(self.repo.is_owned_by(application_id, account_id).await?)
    }

    /// Randomly change application's client secret, and return it.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rotate_client_secret(
        &self,
        application_id: ApplicationId,
    ) -> ServiceResult<String> {
        let client_secret = client_secret::random_client_secret();
        let client_secret_hash = client_secret::hash_client_secret(&client_secret);

        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .set_client_secret_hash(application_id, &client_secret_hash)
            .await?;
        transaction.commit().await?;

        Ok(client_secret)
    }

    /// Rename the application.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rename(&self, application_id: ApplicationId, name: &str) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction.set_name(application_id, name).await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Add a new allowed redirect URL.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_redirect_url(
        &self,
        application_id: ApplicationId,
        redirect_url: &str,
    ) -> ServiceResult<()> {
        // Best-effort early fail: Repository should also check this.
        if self.repo.count_redirect_urls(application_id).await?
            >= limits::application_repo::MAXIMUM_REDIRECT_URL_COUNT
        {
            return Err(ServiceError::TooManyRedirectUrls(
                limits::application_repo::MAXIMUM_REDIRECT_URL_COUNT,
            ));
        }

        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .insert_redirect_url(application_id, redirect_url)
            .await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Remove a redirect URL.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn remove_redirect_url(
        &self,
        application_id: ApplicationId,
        redirect_url: &str,
    ) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .delete_redirect_url(application_id, redirect_url)
            .await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Get the list of allowed redirect URLs.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn list_redirect_urls(
        &self,
        application_id: ApplicationId,
    ) -> ServiceResult<Vec<String>> {
        Ok(self.repo.list_redirect_urls(application_id).await?)
    }
}
