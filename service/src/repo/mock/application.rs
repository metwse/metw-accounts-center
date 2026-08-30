use super::super::{ApplicationRepo, ApplicationRepoTransaction, RepoResult};
use crate::{
    dto, entity,
    id::{AccountId, ApplicationId},
    repo::{RepoError, limits},
};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard};

/// Mock application repository implementation
#[derive(Default)]
pub struct MockApplicationRepoImpl {
    state: Arc<Mutex<ApplicationRepoState>>,
}

#[derive(Default, Clone)]
pub struct ApplicationRepoState {
    pub applications: HashMap<ApplicationId, entity::Application>,
    pub application_redirect_urls: HashMap<ApplicationId, Vec<String>>,
}

impl MockApplicationRepoImpl {
    /// Create a new mock repository.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self::default())
    }

    async fn lock_state(&self) -> MutexGuard<'_, ApplicationRepoState> {
        self.state.lock().await
    }
}

#[async_trait]
impl ApplicationRepo for MockApplicationRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn ApplicationRepoTransaction>> {
        let state = Arc::clone(&self.state).lock_owned().await;

        Ok(Box::new(MockApplicationRepoTransactionImpl {
            state: (*state).clone(),
            commit_state: state,
        }))
    }

    async fn list_by_owner(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Vec<dto::repo::OwnedApplicationSummary>> {
        let state = self.lock_state().await;

        Ok(state
            .applications
            .values()
            .filter(|application_entity| application_entity.owner_account_id == account_id)
            .map(|application_entity| dto::repo::OwnedApplicationSummary {
                application_id: application_entity.application_id,
                name: application_entity.name.clone(),
            })
            .collect())
    }

    async fn list_redirect_urls(&self, application_id: ApplicationId) -> RepoResult<Vec<String>> {
        let state = self.lock_state().await;

        Ok(state
            .application_redirect_urls
            .get(&application_id)
            .cloned()
            .unwrap_or(Vec::new()))
    }

    async fn is_owned_by(
        &self,
        application_id: ApplicationId,
        account_id: AccountId,
    ) -> RepoResult<bool> {
        let state = self.lock_state().await;

        let Some(application_entity) = state.applications.get(&application_id) else {
            return Ok(false);
        };

        Ok(application_entity.owner_account_id == account_id)
    }

    async fn count_by_owner(&self, account_id: AccountId) -> RepoResult<usize> {
        let state = self.lock_state().await;

        Ok(state
            .applications
            .iter()
            .filter(|(_, application_entity)| application_entity.owner_account_id == account_id)
            .count())
    }

    async fn count_redirect_urls(&self, application_id: ApplicationId) -> RepoResult<usize> {
        let state = self.lock_state().await;

        Ok(state
            .application_redirect_urls
            .get(&application_id)
            .map(|redirect_urls| redirect_urls.len())
            .unwrap_or(0))
    }
}

struct MockApplicationRepoTransactionImpl {
    state: ApplicationRepoState,
    commit_state: OwnedMutexGuard<ApplicationRepoState>,
}

#[async_trait]
impl ApplicationRepoTransaction for MockApplicationRepoTransactionImpl {
    async fn commit(mut self: Box<Self>) -> RepoResult<()> {
        *self.commit_state = self.state;

        Ok(())
    }

    async fn insert(&mut self, application_entity: entity::Application) -> RepoResult<()> {
        // Silently skip if the owner reached maximum allowed application
        // count.
        if self
            .state
            .applications
            .iter()
            .filter(|(_, db_application_entity)| {
                db_application_entity.owner_account_id == application_entity.owner_account_id
            })
            .count()
            < limits::application_repo::MAXIMUM_APPLICATION_COUNT
        {
            self.state
                .applications
                .insert(application_entity.application_id, application_entity);
        }

        Ok(())
    }

    async fn delete(&mut self, application_id: ApplicationId) -> RepoResult<()> {
        self.state.applications.remove(&application_id);
        self.state.application_redirect_urls.remove(&application_id);

        Ok(())
    }

    async fn set_client_secret_hash(
        &mut self,
        application_id: ApplicationId,
        client_secret_hash: &[u8; 32],
    ) -> RepoResult<()> {
        if let Some(application_entity) = self.state.applications.get_mut(&application_id) {
            application_entity.client_secret_hash = client_secret_hash.to_owned();
        }

        Ok(())
    }

    async fn set_name(&mut self, application_id: ApplicationId, name: &str) -> RepoResult<()> {
        if let Some(application_entity) = self.state.applications.get_mut(&application_id) {
            application_entity.name = name.to_owned();
        }

        Ok(())
    }

    async fn insert_redirect_url(
        &mut self,
        application_id: ApplicationId,
        redirect_url: &str,
    ) -> RepoResult<()> {
        let redirect_urls = self
            .state
            .application_redirect_urls
            .entry(application_id)
            .or_default();

        let redirect_url = redirect_url.to_string();

        // Silently skip if the application has reached maximum redirect URL
        // count.
        if redirect_urls.len() >= limits::application_repo::MAXIMUM_REDIRECT_URL_COUNT {
            Ok(())
        } else if redirect_urls.contains(&redirect_url) {
            Err(RepoError::Internal(
                "redirect url exists in the application",
            ))
        } else {
            redirect_urls.push(redirect_url);

            Ok(())
        }
    }

    async fn delete_redirect_url(
        &mut self,
        application_id: ApplicationId,
        redirect_url: &str,
    ) -> RepoResult<()> {
        let redirect_urls = self
            .state
            .application_redirect_urls
            .entry(application_id)
            .or_default();

        if let Some(index) = redirect_urls
            .iter()
            .position(|redirect_url_in_vec| redirect_url_in_vec == redirect_url)
        {
            redirect_urls.remove(index);
        };

        Ok(())
    }
}
