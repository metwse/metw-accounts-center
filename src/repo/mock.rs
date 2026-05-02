use super::{AccountRepo, AccountRepoTransaction, RepoResult};
use crate::{dto, entity};
use async_trait::async_trait;
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard};

/// Mock account repository implementatoin
pub struct AccountRepoImpl {
    state: Arc<Mutex<State>>,
}

/// Create a new mock repository.
pub fn new_mock() -> Box<dyn AccountRepo> {
    Box::new(AccountRepoImpl {
        state: Arc::new(Mutex::new(State::default())),
    })
}

impl AccountRepoImpl {
    async fn lock_state(&self) -> MutexGuard<'_, State> {
        self.state.lock().await
    }
}

#[async_trait]
impl AccountRepo for AccountRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AccountRepoTransaction>> {
        Ok(Box::new(AccountRepoTransactionImpl {
            state: Arc::clone(&self.state).lock_owned().await,
        }))
    }

    async fn get_login_by_email(&self, email: &str) -> RepoResult<Option<dto::repo::Login>> {
        let state = self.lock_state().await;

        if let Some(email_entity) = state.emails.get(email) {
            Ok(Some(dto::repo::Login {
                id: email_entity.account_id,
                password_hash: state.accounts[&email_entity.account_id]
                    .password_hash
                    .clone(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_login_by_username(&self, username: &str) -> RepoResult<Option<dto::repo::Login>> {
        let state = self.lock_state().await;

        if let Some(username_entity) = state.usernames.get(username) {
            Ok(Some(dto::repo::Login {
                id: username_entity.account_id,
                password_hash: state.accounts[&username_entity.account_id]
                    .password_hash
                    .clone(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_primary_username(&self, _id: entity::AccountId) -> RepoResult<Option<String>> {
        todo!()
    }

    async fn get_nonexpiring_username_aliases(
        &self,
        _id: entity::AccountId,
    ) -> RepoResult<Vec<String>> {
        todo!()
    }

    async fn get_primary_email(&self, _id: entity::AccountId) -> RepoResult<Option<String>> {
        todo!()
    }

    async fn get_secondary_emails(&self, _id: entity::AccountId) -> RepoResult<Vec<String>> {
        todo!()
    }

    async fn get_keys(&self, _id: entity::AccountId) -> RepoResult<Option<dto::repo::Keys>> {
        todo!()
    }
}

#[derive(Default)]
struct State {
    accounts: HashMap<entity::AccountId, entity::Account>,
    emails: HashMap<String, entity::Email>,
    usernames: HashMap<String, entity::Username>,
}

struct AccountRepoTransactionImpl {
    state: OwnedMutexGuard<State>,
}

#[async_trait]
impl AccountRepoTransaction for AccountRepoTransactionImpl {
    async fn commit(self: Box<Self>) -> RepoResult<()> {
        Ok(())
    }

    async fn upsert_account(
        &mut self,
        id: entity::AccountId,
        password_hash: &str,
        keys: &dto::repo::Keys,
    ) -> RepoResult<()> {
        let account_entity = self.state.accounts.entry(id).or_default();

        account_entity.id = id;

        account_entity.password_hash = password_hash.to_string();
        account_entity.identity_key = keys.identity_key.clone();
        account_entity.encrpyted_private_key = keys.encrypted_private_key.clone();
        account_entity.encrpyted_master_key = keys.encrypted_master_key.clone();

        Ok(())
    }

    async fn add_email(&mut self, id: entity::AccountId, email: &str) -> RepoResult<bool> {
        if self.state.emails.contains_key(email) {
            Ok(false)
        } else {
            self.state.emails.insert(
                email.to_string(),
                entity::Email {
                    email: email.to_string(),
                    account_id: id,
                    is_primary: false,
                    created_at: Utc::now(),
                },
            );

            Ok(true)
        }
    }

    async fn add_username(&mut self, id: entity::AccountId, username: &str) -> RepoResult<bool> {
        if self.state.usernames.contains_key(username) {
            Ok(false)
        } else {
            self.state.usernames.insert(
                username.to_string(),
                entity::Username {
                    username: username.to_string(),
                    account_id: id,
                    is_primary: false,
                    created_at: Utc::now(),
                    expires_at: None,
                },
            );

            Ok(true)
        }
    }

    async fn set_primary_email(&mut self, email: &str, is_primary: bool) -> RepoResult<bool> {
        if let Some(email) = self.state.emails.get_mut(email) {
            email.is_primary = is_primary;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn set_primary_username(&mut self, username: &str, is_primary: bool) -> RepoResult<bool> {
        if let Some(username) = self.state.usernames.get_mut(username) {
            username.is_primary = is_primary;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
