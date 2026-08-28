use super::super::{AppRepo, AppRepoTransaction, RepoResult};
use crate::{
    dto, entity,
    id::{AccountId, AppId},
    repo::RepoError,
};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard};

/// Mock application repository implementation
#[derive(Default)]
pub struct MockAppRepoImpl {
    state: Arc<Mutex<AppRepoState>>,
}

#[derive(Default, Clone)]
pub struct AppRepoState {
    pub apps: HashMap<AppId, entity::App>,
    pub app_redirect_urls: HashMap<AppId, Vec<String>>,
}

impl MockAppRepoImpl {
    /// Create a new mock repository.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self::default())
    }

    async fn lock_state(&self) -> MutexGuard<'_, AppRepoState> {
        self.state.lock().await
    }
}

#[async_trait]
impl AppRepo for MockAppRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AppRepoTransaction>> {
        let state = Arc::clone(&self.state).lock_owned().await;

        Ok(Box::new(MockAppRepoTransactionImpl {
            state: (*state).clone(),
            commit_state: state,
        }))
    }

    async fn get_apps(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Vec<dto::repo::OwnedBasicAppInfo>> {
        let state = self.lock_state().await;

        Ok(state
            .apps
            .values()
            .filter(|app_entity| app_entity.owner_account_id == account_id)
            .map(|app_entity| dto::repo::OwnedBasicAppInfo {
                app_id: app_entity.app_id,
                name: app_entity.name.clone(),
            })
            .collect())
    }

    async fn get_redirect_urls(&self, app_id: AppId) -> RepoResult<Vec<String>> {
        let state = self.lock_state().await;

        Ok(state
            .app_redirect_urls
            .get(&app_id)
            .cloned()
            .unwrap_or(Vec::new()))
    }

    async fn is_app_owned_by(&self, account_id: AccountId, app_id: AppId) -> RepoResult<bool> {
        let state = self.lock_state().await;

        let Some(app_entity) = state.apps.get(&app_id) else {
            return Ok(false);
        };

        Ok(app_entity.owner_account_id == account_id)
    }
}

struct MockAppRepoTransactionImpl {
    state: AppRepoState,
    commit_state: OwnedMutexGuard<AppRepoState>,
}

#[async_trait]
impl AppRepoTransaction for MockAppRepoTransactionImpl {
    async fn commit(mut self: Box<Self>) -> RepoResult<()> {
        *self.commit_state = self.state;

        Ok(())
    }

    async fn insert_app(&mut self, app: entity::App) -> RepoResult<()> {
        self.state.apps.insert(app.app_id, app);

        Ok(())
    }

    async fn update_client_secret_hash(
        &mut self,
        app_id: AppId,
        client_secret_hash: &[u8; 32],
    ) -> RepoResult<()> {
        if let Some(app_entity) = self.state.apps.get_mut(&app_id) {
            app_entity.client_secret_hash = client_secret_hash.to_owned();
        }

        Ok(())
    }

    /// Deletes the application.
    async fn delete_app(&mut self, app_id: AppId) -> RepoResult<()> {
        self.state.apps.remove(&app_id);
        self.state.app_redirect_urls.remove(&app_id);

        Ok(())
    }

    /// Adds a new redirect URL.
    async fn add_redirect_url(&mut self, app_id: AppId, redirect_url: &str) -> RepoResult<()> {
        let redirect_urls = self.state.app_redirect_urls.entry(app_id).or_default();

        let redirect_url = redirect_url.to_string();

        if redirect_urls.contains(&redirect_url) {
            Err(RepoError::Internal(
                "redirect url exists in the application",
            ))
        } else {
            redirect_urls.push(redirect_url);

            Ok(())
        }
    }

    /// Removes the redirect URL.
    async fn remove_redirect_url(&mut self, app_id: AppId, redirect_url: &str) -> RepoResult<()> {
        let redirect_urls = self.state.app_redirect_urls.entry(app_id).or_default();

        if let Some(index) = redirect_urls
            .iter()
            .position(|redirect_url_in_vec| redirect_url_in_vec == redirect_url)
        {
            redirect_urls.remove(index);
        };

        Ok(())
    }
}
