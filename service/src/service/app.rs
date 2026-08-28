use super::ServiceResult;
use crate::{
    dto, entity,
    id::{AccountId, AppId},
    repo::AppRepo,
    util::client_secret,
};

/// User-registered app state.
pub struct AppService {
    repo: Box<dyn AppRepo>,
}

impl AppService {
    /// Creates a new account service.
    pub fn new(repo: Box<dyn AppRepo>) -> Self {
        Self { repo }
    }

    /// Register a new application, and return its client secret and ID.
    pub async fn create_app(
        &self,
        account_id: AccountId,
        name: &str,
    ) -> ServiceResult<dto::service::NewApp> {
        let app_id = AppId::unique();
        let client_secret = client_secret::random_client_secret();
        let client_secret_hash = client_secret::hash_client_secret(&client_secret);

        let app = entity::App {
            app_id,
            owner_account_id: account_id,
            name: name.to_string(),
            client_secret_hash,
        };

        let mut transaction = self.repo.begin_transaction().await?;
        transaction.insert_app(app).await?;
        transaction.commit().await?;

        Ok(dto::service::NewApp {
            app_id,
            client_secret,
        })
    }

    /// Get the list of registered applications.
    pub async fn get_apps(
        &self,
        account_id: AccountId,
    ) -> ServiceResult<Vec<dto::repo::OwnedBasicAppInfo>> {
        Ok(self.repo.get_apps(account_id).await?)
    }

    /// Check app ownership.
    pub async fn is_app_owned_by(
        &self,
        account_id: AccountId,
        app_id: AppId,
    ) -> ServiceResult<bool> {
        Ok(self.repo.is_app_owned_by(account_id, app_id).await?)
    }

    /// Randomly change application's client secret, and return it.
    pub async fn roll_client_secret(&self, app_id: AppId) -> ServiceResult<String> {
        let client_secret = client_secret::random_client_secret();
        let client_secret_hash = client_secret::hash_client_secret(&client_secret);

        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .update_client_secret_hash(app_id, &client_secret_hash)
            .await?;
        transaction.commit().await?;

        Ok(client_secret)
    }

    /// Delete the application.
    pub async fn delete_app(&self, app_id: AppId) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction.delete_app(app_id).await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Add a new allowed redirect URL.
    pub async fn add_redirect_url(&self, app_id: AppId, redirect_url: &str) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction.add_redirect_url(app_id, redirect_url).await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Remove an redirect URL.
    pub async fn remove_redirect_url(
        &self,
        app_id: AppId,
        redirect_url: &str,
    ) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .remove_redirect_url(app_id, redirect_url)
            .await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Get the list of allowed redirect URLs.
    pub async fn get_redirect_urls(&self, app_id: AppId) -> ServiceResult<Vec<String>> {
        Ok(self.repo.get_redirect_urls(app_id).await?)
    }
}
