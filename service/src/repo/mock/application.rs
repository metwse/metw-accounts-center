use super::super::{ApplicationRepo, ApplicationRepoTransaction, RepoResult};
use crate::{
    dto, entity,
    id::{AccountId, ApplicationId},
    repo::RepoError,
};
use async_trait::async_trait;
use metw_id::checked_now;
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
    pub account_application_consents:
        HashMap<(AccountId, ApplicationId), entity::AccountApplicationConsent>,
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

    async fn list_consents(
        &self,
        account_id: AccountId,
        after_application_id: Option<ApplicationId>,
    ) -> RepoResult<Vec<dto::repo::OwnedApplicationAccountConsent>> {
        let state = self.lock_state().await;

        let mut consents: Vec<dto::repo::OwnedApplicationAccountConsent> = state
            .account_application_consents
            .iter()
            .filter(|((db_account_id, db_application_id), _)| {
                *db_account_id == account_id
                    && after_application_id
                        .map(|after| *db_application_id > after)
                        .unwrap_or(true)
            })
            .map(|((_, application_id), consent_entity)| {
                dto::repo::OwnedApplicationAccountConsent {
                    application_id: *application_id,
                    name: state.applications[application_id].name.clone(),

                    created_at: consent_entity.created_at,
                }
            })
            .collect();

        consents.sort_by_key(|consent| consent.application_id);
        consents.truncate(10);

        Ok(consents)
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

    async fn consent_exists(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> RepoResult<bool> {
        let state = self.lock_state().await;

        Ok(state
            .account_application_consents
            .contains_key(&(account_id, application_id)))
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

    async fn lock(&mut self, _application_id: ApplicationId) -> RepoResult<()> {
        // The mock implementation already locks entire database.
        Ok(())
    }

    async fn lock_account(&mut self, _account_id: AccountId) -> RepoResult<()> {
        // The mock implementation already locks entire database.
        Ok(())
    }

    async fn insert(&mut self, application_entity: entity::Application) -> RepoResult<()> {
        self.state
            .applications
            .insert(application_entity.application_id, application_entity);

        Ok(())
    }

    async fn delete(&mut self, application_id: ApplicationId) -> RepoResult<()> {
        self.state.applications.remove(&application_id);
        self.state.application_redirect_urls.remove(&application_id);

        let keys_to_remove: Vec<_> = self
            .state
            .account_application_consents
            .keys()
            .filter(|(_, db_application_id)| *db_application_id != application_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.state.account_application_consents.remove_entry(&key);
        }

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

    async fn insert_consent(
        &mut self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> RepoResult<()> {
        if self
            .state
            .account_application_consents
            .contains_key(&(account_id, application_id))
        {
            // Silently ignore if consent already exists.
            return Ok(());
        }

        self.state.account_application_consents.insert(
            (account_id, application_id),
            entity::AccountApplicationConsent {
                account_id,
                application_id,
                created_at: checked_now(),
                key_encryption_algorithm: entity::KeyEncryptionAlgorithm::None,
                master_key_encrypted_key: None,
                master_key_id: None,
            },
        );

        Ok(())
    }

    async fn delete_consent(
        &mut self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> RepoResult<()> {
        self.state
            .account_application_consents
            .remove(&(account_id, application_id));

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

        if redirect_urls.contains(&redirect_url) {
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

    async fn count_by_owner(&mut self, account_id: AccountId) -> RepoResult<usize> {
        Ok(self
            .state
            .applications
            .iter()
            .filter(|(_, application_entity)| application_entity.owner_account_id == account_id)
            .count())
    }

    async fn count_redirect_urls(&mut self, application_id: ApplicationId) -> RepoResult<usize> {
        Ok(self
            .state
            .application_redirect_urls
            .get(&application_id)
            .map(|redirect_urls| redirect_urls.len())
            .unwrap_or(0))
    }
}
