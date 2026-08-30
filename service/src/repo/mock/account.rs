use super::super::{AccountRepo, AccountRepoTransaction, RepoResult};
use crate::{checked_now, dto, entity, id::AccountId, repo::RepoError};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard};

/// Mock account repository implementation
#[derive(Default)]
pub struct MockAccountRepoImpl {
    state: Arc<Mutex<AccountRepoState>>,
}

#[derive(Default, Clone)]
struct AccountRepoState {
    accounts: HashMap<AccountId, entity::Account>,
    emails: HashMap<String, entity::Email>,
    usernames: HashMap<String, entity::Username>,
    account_flags: HashMap<AccountId, entity::AccountFlags>,
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
        let state = Arc::clone(&self.state).lock_owned().await;

        Ok(Box::new(MockAccountRepoTransactionImpl {
            state: (*state).clone(),
            commit_state: state,
        }))
    }

    async fn get_login_credentials_by_account_id(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        let state = self.lock_state().await;

        if let Some(account) = state.accounts.get(&account_id) {
            Ok(Some(dto::repo::OwnedLoginCredentials {
                account_id,
                password_hash: account.password_hash.clone(),
                is_email_verified: state.account_flags[&account_id].is_email_verified,
                server_password_hash_algorithm: account.server_password_hash_algorithm.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_login_credentials_by_email(
        &self,
        email: &str,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        let state = self.lock_state().await;

        if let Some(email_entity) = state.emails.get(email) {
            Ok(Some(dto::repo::OwnedLoginCredentials {
                account_id: email_entity.account_id,
                password_hash: state.accounts[&email_entity.account_id]
                    .password_hash
                    .clone(),
                is_email_verified: true,
                server_password_hash_algorithm: state.accounts[&email_entity.account_id]
                    .server_password_hash_algorithm
                    .clone(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_login_credentials_by_username(
        &self,
        username: &str,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        let state = self.lock_state().await;

        if let Some(username_entity) = state.usernames.get(username)
            && username_entity.expires_at.is_none()
        {
            Ok(Some(dto::repo::OwnedLoginCredentials {
                account_id: username_entity.account_id,
                password_hash: state.accounts[&username_entity.account_id]
                    .password_hash
                    .clone(),
                is_email_verified: state
                    .account_flags
                    .get(&username_entity.account_id)
                    .unwrap()
                    .is_email_verified,
                server_password_hash_algorithm: state.accounts[&username_entity.account_id]
                    .server_password_hash_algorithm
                    .clone(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_client_password_kdf_by_email(
        &self,
        email: &str,
    ) -> RepoResult<Option<entity::ClientPasswordKdf>> {
        let state = self.lock_state().await;

        if let Some(email_entity) = state.emails.get(email) {
            Ok(Some(
                state.accounts[&email_entity.account_id]
                    .client_password_kdf
                    .clone(),
            ))
        } else {
            Ok(None)
        }
    }

    async fn get_client_password_kdf_by_username(
        &self,
        username: &str,
    ) -> RepoResult<Option<entity::ClientPasswordKdf>> {
        let state = self.lock_state().await;

        if let Some(username_entity) = state.usernames.get(username)
            && username_entity.expires_at.is_none()
        {
            Ok(Some(
                state.accounts[&username_entity.account_id]
                    .client_password_kdf
                    .clone(),
            ))
        } else {
            Ok(None)
        }
    }

    async fn get_client_password_kdf_by_account_id(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Option<entity::ClientPasswordKdf>> {
        let state = self.lock_state().await;

        if let Some(account_entity) = state.accounts.get(&account_id) {
            Ok(Some(account_entity.client_password_kdf.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_primary_username(&self, account_id: AccountId) -> RepoResult<Option<String>> {
        let state = self.lock_state().await;

        for username_entity in state.usernames.values() {
            if username_entity.is_primary && username_entity.account_id == account_id {
                return Ok(Some(username_entity.username.clone()));
            }
        }

        Ok(None)
    }

    async fn get_nonexpiring_username_aliases(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Vec<String>> {
        let state = self.lock_state().await;

        let mut nonexpiring_usernames = Vec::new();

        for username_entity in state.usernames.values() {
            if username_entity.expires_at.is_none()
                && !username_entity.is_primary
                && username_entity.account_id == account_id
            {
                nonexpiring_usernames.push(username_entity.username.clone());
            }
        }

        Ok(nonexpiring_usernames)
    }

    async fn get_primary_email(&self, account_id: AccountId) -> RepoResult<Option<String>> {
        let state = self.lock_state().await;

        for email_entity in state.emails.values() {
            if email_entity.is_primary && email_entity.account_id == account_id {
                return Ok(Some(email_entity.email.clone()));
            }
        }

        Ok(None)
    }

    async fn get_secondary_emails(&self, account_id: AccountId) -> RepoResult<Vec<String>> {
        let state = self.lock_state().await;

        let mut secondary_emails = Vec::new();

        for email_entity in state.emails.values() {
            if email_entity.account_id == account_id && !email_entity.is_primary {
                secondary_emails.push(email_entity.email.clone());
            }
        }

        Ok(secondary_emails)
    }

    async fn compare_and_set_primary_email(
        &self,
        account_id: AccountId,
        expected_primary_email: &str,
        new_primary_email: &str,
    ) -> RepoResult<bool> {
        let mut state = self.lock_state().await;

        {
            let Some(current_primary_email_entity) = state.emails.get(expected_primary_email)
            else {
                return Ok(false);
            };
            let Some(new_primary_email_entity) = state.emails.get(new_primary_email) else {
                return Ok(false);
            };

            if !(current_primary_email_entity.is_primary
                && current_primary_email_entity.account_id == account_id
                && new_primary_email_entity.account_id == account_id
                && new_primary_email != expected_primary_email)
            {
                return Ok(false);
            }
        }

        state
            .emails
            .get_mut(expected_primary_email)
            .unwrap()
            .is_primary = false;
        state.emails.get_mut(new_primary_email).unwrap().is_primary = true;

        Ok(true)
    }

    async fn delete_email_if_not_primary(
        &self,
        account_id: AccountId,
        email: &str,
    ) -> RepoResult<bool> {
        let mut state = self.lock_state().await;

        let Some(email_entity) = state.emails.get(email) else {
            return Ok(false);
        };

        if email_entity.account_id == account_id && !email_entity.is_primary {
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

    async fn is_email_owned_by(&self, account_id: AccountId, email: &str) -> RepoResult<bool> {
        let state = self.lock_state().await;

        let Some(email_entity) = state.emails.get(email) else {
            return Ok(false);
        };

        Ok(email_entity.account_id == account_id)
    }
}

struct MockAccountRepoTransactionImpl {
    state: AccountRepoState,
    commit_state: OwnedMutexGuard<AccountRepoState>,
}

#[async_trait]
impl AccountRepoTransaction for MockAccountRepoTransactionImpl {
    async fn commit(mut self: Box<Self>) -> RepoResult<()> {
        *self.commit_state = self.state;

        Ok(())
    }

    async fn lock(&mut self, _account_id: AccountId) -> RepoResult<()> {
        // The mock implementation already locks entire database.
        Ok(())
    }

    async fn insert(&mut self, account_entity: &entity::Account) -> RepoResult<()> {
        self.state
            .accounts
            .insert(account_entity.account_id, account_entity.clone());

        Ok(())
    }

    async fn insert_default_flags(&mut self, account_id: AccountId) -> RepoResult<()> {
        self.state.account_flags.insert(
            account_id,
            entity::AccountFlags {
                account_id,
                is_email_verified: false,
            },
        );

        Ok(())
    }

    async fn insert_email(
        &mut self,
        account_id: AccountId,
        email: &str,
        is_primary: bool,
    ) -> RepoResult<()> {
        if self.state.emails.contains_key(email) {
            Err(RepoError::Internal("email is taken"))
        } else {
            self.state.emails.insert(
                email.to_string(),
                entity::Email {
                    email: email.to_string(),
                    account_id,
                    is_primary,
                    created_at: checked_now(),
                },
            );

            Ok(())
        }
    }

    async fn insert_username(
        &mut self,
        account_id: AccountId,
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
                    account_id,
                    is_primary,
                    created_at: checked_now(),
                    expires_at: None,
                },
            );

            Ok(())
        }
    }

    async fn set_email_verified_flag(
        &mut self,
        account_id: AccountId,
        is_email_verified: bool,
    ) -> RepoResult<()> {
        if let Some(account_flags_entity) = self.state.account_flags.get_mut(&account_id) {
            account_flags_entity.is_email_verified = is_email_verified;

            Ok(())
        } else {
            Err(RepoError::Internal("account does not exists"))
        }
    }

    async fn set_password_credentials(
        &mut self,
        account_id: AccountId,
        password_hash: &str,
        client_password_kdf: &entity::ClientPasswordKdf,
        server_password_hash_algorithm: &entity::ServerPasswordHashAlgorithm,
    ) -> RepoResult<()> {
        if let Some(account_entity) = self.state.accounts.get_mut(&account_id) {
            account_entity.password_hash = Some(password_hash.into());
            account_entity.client_password_kdf = client_password_kdf.clone();
            account_entity.server_password_hash_algorithm = server_password_hash_algorithm.clone();

            Ok(())
        } else {
            Err(RepoError::Internal("account does not exists"))
        }
    }

    async fn count_emails(&mut self, account_id: AccountId) -> RepoResult<usize> {
        Ok(self
            .state
            .emails
            .iter()
            .filter(|(_, email_entity)| email_entity.account_id == account_id)
            .count())
    }
}
