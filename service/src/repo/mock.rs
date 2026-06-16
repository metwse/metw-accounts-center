use super::{AccountRepo, AccountRepoTransaction, RepoResult, TokenRepo};
use crate::{dto, entity, id::AccountId, repo::RepoError};
use async_trait::async_trait;
use chrono::Utc;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard};

/// Mock account repository implementation
#[derive(Default)]
pub struct MockAccountRepoImpl {
    state: Arc<Mutex<AccountRepoState>>,
}

impl MockAccountRepoImpl {
    /// Create a new mock repository.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self::default())
    }

    async fn lock_state(&self) -> MutexGuard<'_, AccountRepoState> {
        self.state.lock().await
    }
}

#[async_trait]
impl AccountRepo for MockAccountRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AccountRepoTransaction>> {
        Ok(Box::new(MockAccountRepoTransactionImpl {
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

        if let Some(username_entity) = state.usernames.get(username)
            && username_entity.expires_at.is_none()
        {
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

    async fn get_primary_username(&self, id: AccountId) -> RepoResult<Option<String>> {
        let state = self.lock_state().await;

        for username_entity in state.usernames.values() {
            if username_entity.is_primary && username_entity.account_id == id {
                return Ok(Some(username_entity.username.clone()));
            }
        }

        Ok(None)
    }

    async fn get_nonexpiring_username_aliases(&self, id: AccountId) -> RepoResult<Vec<String>> {
        let state = self.lock_state().await;

        let mut nonexpiring_usernames = Vec::new();

        for username_entity in state.usernames.values() {
            if username_entity.expires_at.is_none()
                && !username_entity.is_primary
                && username_entity.account_id == id
            {
                nonexpiring_usernames.push(username_entity.username.clone());
            }
        }

        Ok(nonexpiring_usernames)
    }

    async fn get_primary_email(&self, id: AccountId) -> RepoResult<Option<String>> {
        let state = self.lock_state().await;

        for email_entity in state.emails.values() {
            if email_entity.is_primary && email_entity.account_id == id {
                return Ok(Some(email_entity.email.clone()));
            }
        }

        Ok(None)
    }

    async fn get_secondary_emails(&self, id: AccountId) -> RepoResult<Vec<String>> {
        let state = self.lock_state().await;

        let mut secondary_emails = Vec::new();

        for email_entity in state.emails.values() {
            if email_entity.account_id == id && !email_entity.is_primary {
                secondary_emails.push(email_entity.email.clone());
            }
        }

        Ok(secondary_emails)
    }

    async fn get_keys(&self, id: AccountId) -> RepoResult<Option<dto::repo::Keys>> {
        let state = self.lock_state().await;

        if let Some(account_entity) = state.accounts.get(&id) {
            Ok(Some(dto::repo::Keys {
                identity_key: account_entity.identity_key.clone(),
                encrypted_private_key: account_entity.encrypted_private_key.clone(),
                encrypted_master_key: account_entity.encrypted_master_key.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_account_flags(&self, id: AccountId) -> RepoResult<Option<entity::AccountFlags>> {
        let state = self.lock_state().await;

        if let Some(flags_entity) = state.account_flags.get(&id) {
            Ok(Some(flags_entity.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set_primary_email_if_current_is(
        &self,
        id: AccountId,
        current_primary_email: &str,
        new_primary_email: &str,
    ) -> RepoResult<bool> {
        let mut state = self.lock_state().await;

        {
            let Some(current_primary_email_entity) = state.emails.get(current_primary_email) else {
                return Ok(false);
            };
            let Some(new_primary_email_entity) = state.emails.get(new_primary_email) else {
                return Ok(false);
            };

            if !(current_primary_email_entity.is_primary
                && current_primary_email_entity.account_id == id
                && new_primary_email_entity.account_id == id
                && new_primary_email != current_primary_email)
            {
                return Ok(false);
            }
        }

        state
            .emails
            .get_mut(current_primary_email)
            .unwrap()
            .is_primary = false;
        state.emails.get_mut(new_primary_email).unwrap().is_primary = true;

        Ok(true)
    }

    async fn remove_email_if_not_primary(&self, id: AccountId, email: &str) -> RepoResult<bool> {
        let mut state = self.lock_state().await;

        let Some(email_entity) = state.emails.get(email) else {
            return Ok(false);
        };

        if email_entity.account_id == id && !email_entity.is_primary {
            state.emails.remove(email);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn is_username_taken(&self, username: &str) -> RepoResult<bool> {
        let state = self.lock_state().await;

        Ok(state.usernames.contains_key(username))
    }

    async fn is_email_taken(&self, email: &str) -> RepoResult<bool> {
        let state = self.lock_state().await;

        Ok(state.emails.contains_key(email))
    }

    async fn is_email_taken_by(&self, id: AccountId, email: &str) -> RepoResult<bool> {
        let state = self.lock_state().await;

        let Some(email_entity) = state.emails.get(email) else {
            return Ok(false);
        };

        Ok(email_entity.account_id == id)
    }
}

#[derive(Default)]
struct AccountRepoState {
    accounts: HashMap<AccountId, entity::Account>,
    emails: HashMap<String, entity::Email>,
    usernames: HashMap<String, entity::Username>,
    account_flags: HashMap<AccountId, entity::AccountFlags>,
}

struct MockAccountRepoTransactionImpl {
    state: OwnedMutexGuard<AccountRepoState>,
}

#[async_trait]
impl AccountRepoTransaction for MockAccountRepoTransactionImpl {
    async fn commit(self: Box<Self>) -> RepoResult<()> {
        Ok(())
    }

    async fn upsert_account(
        &mut self,
        id: AccountId,
        password_hash: &str,
        keys: &dto::repo::Keys,
    ) -> RepoResult<()> {
        let account_entity = self.state.accounts.entry(id).or_default();

        account_entity.id = id;

        account_entity.password_hash = password_hash.to_string();
        account_entity.identity_key = keys.identity_key.clone();
        account_entity.encrypted_private_key = keys.encrypted_private_key.clone();
        account_entity.encrypted_master_key = keys.encrypted_master_key.clone();

        Ok(())
    }

    async fn insert_default_flags(&mut self, id: AccountId) -> RepoResult<()> {
        self.state.account_flags.insert(
            id,
            entity::AccountFlags {
                id,
                is_verified: false,
            },
        );

        Ok(())
    }

    async fn add_email(&mut self, id: AccountId, email: &str, is_primary: bool) -> RepoResult<()> {
        if self.state.emails.contains_key(email) {
            Err(RepoError::Internal("email is taken"))
        } else {
            self.state.emails.insert(
                email.to_string(),
                entity::Email {
                    email: email.to_string(),
                    account_id: id,
                    is_primary,
                    created_at: Utc::now(),
                },
            );

            Ok(())
        }
    }

    async fn add_username(
        &mut self,
        id: AccountId,
        username: &str,
        is_primary: bool,
    ) -> RepoResult<()> {
        if self.state.usernames.contains_key(username) {
            Err(RepoError::Internal("username is taken"))
        } else {
            self.state.usernames.insert(
                username.to_string(),
                entity::Username {
                    username: username.to_string(),
                    account_id: id,
                    is_primary,
                    created_at: Utc::now(),
                    expires_at: None,
                },
            );

            Ok(())
        }
    }

    async fn set_verified_flag(&mut self, id: AccountId, is_verified: bool) -> RepoResult<()> {
        if let Some(account_flags_entity) = self.state.account_flags.get_mut(&id) {
            account_flags_entity.is_verified = is_verified;

            Ok(())
        } else {
            Err(RepoError::Internal("account does not exists"))
        }
    }
}

/// Mock token repo implementation.
#[derive(Default)]
pub struct MockTokenRepoImpl {
    revocations: Arc<Mutex<HashSet<Vec<u8>>>>,
}

impl MockTokenRepoImpl {
    /// Create a new mock repository.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self::default())
    }
}

#[async_trait]
impl TokenRepo for MockTokenRepoImpl {
    async fn check_and_revoke(
        &self,
        fingerprint: &[u8],
        revoke_for: std::time::Duration,
    ) -> RepoResult<bool> {
        let mut state = self.revocations.lock().await;

        if state.contains(fingerprint) {
            Ok(true)
        } else {
            state.insert(fingerprint.into());

            tokio::spawn({
                let state = Arc::clone(&self.revocations);
                let fingerprint: Vec<u8> = fingerprint.into();

                async move {
                    tokio::time::sleep(revoke_for).await;

                    let mut state = state.lock().await;

                    state.remove(&fingerprint);
                }
            });

            Ok(false)
        }
    }

    async fn check_revocation(&self, fingerprint: &[u8]) -> RepoResult<bool> {
        Ok(self.revocations.lock().await.contains(fingerprint))
    }
}
