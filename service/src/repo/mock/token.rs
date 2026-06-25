use super::super::{RepoResult, TokenRepo};
use crate::{
    checked_now,
    id::AccountId,
    token::{DecodedToken, TokenScope},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

type ScopeRevocationKey = (AccountId, &'static str);

/// Mock token repo implementation.
#[derive(Default)]
pub struct MockTokenRepoImpl {
    fingerprint_revocations: Arc<Mutex<HashSet<Vec<u8>>>>,
    scope_revocations: Arc<Mutex<HashMap<ScopeRevocationKey, DateTime<Utc>>>>,
    account_revocations: Arc<Mutex<HashMap<AccountId, DateTime<Utc>>>>,
}

impl MockTokenRepoImpl {
    /// Create a new mock repository.
    pub fn boxed_new() -> Box<Self> {
        Box::new(Self::default())
    }
}

#[async_trait]
impl TokenRepo for MockTokenRepoImpl {
    async fn check_and_revoke_token(&self, token: &DecodedToken) -> RepoResult<bool> {
        if self.check_account_revocation(token).await || self.check_scope_revocation(token).await {
            return Ok(true);
        }

        let mut state = self.fingerprint_revocations.lock().await;

        let is_revoked = state.contains(&token.fingerprint);

        if !is_revoked {
            state.insert(token.fingerprint.clone());
            drop(state);

            tokio::spawn({
                let state = Arc::clone(&self.fingerprint_revocations);
                let fingerprint = token.fingerprint.clone();
                let expiration = token.safe_lifetime();

                async move {
                    tokio::time::sleep(expiration).await;
                    let mut state = state.lock().await;
                    state.remove(&fingerprint);
                }
            });
        };

        Ok(is_revoked)
    }

    async fn check_and_revoke_account_tokens_with_scope(
        &self,
        token: &DecodedToken,
    ) -> RepoResult<bool> {
        if self.check_account_revocation(token).await
            || self.check_fingerprint_revocation(token).await
        {
            return Ok(true);
        }

        let mut state = self.scope_revocations.lock().await;

        let key = (token.id, token.scope.scope_name());

        let is_revoked = if let Some(&cutoff_time) = state.get(&key) {
            token.issued_at <= cutoff_time
        } else {
            false
        };

        if !is_revoked {
            self.update_token_cutoff(
                &mut *state,
                self.scope_revocations.clone(),
                key,
                token.scope.safe_scope_lifetime(),
            );
        };

        Ok(is_revoked)
    }

    async fn check_and_revoke_account_tokens(&self, token: &DecodedToken) -> RepoResult<bool> {
        if self.check_scope_revocation(token).await
            || self.check_fingerprint_revocation(token).await
        {
            return Ok(true);
        }

        let mut state = self.account_revocations.lock().await;

        let key = token.id;

        let is_revoked = if let Some(&time) = state.get(&key) {
            token.issued_at <= time
        } else {
            false
        };

        if !is_revoked {
            self.update_token_cutoff(
                &mut *state,
                self.account_revocations.clone(),
                key,
                TokenScope::safe_global_lifetime(),
            );
        };

        Ok(is_revoked)
    }

    async fn revoke_account_tokens_with_scope(
        &self,
        account_id: AccountId,
        scope: &TokenScope,
    ) -> RepoResult<Option<DateTime<Utc>>> {
        let mut state = self.scope_revocations.lock().await;

        Ok(self.update_token_cutoff(
            &mut *state,
            self.scope_revocations.clone(),
            (account_id, scope.scope_name()),
            scope.safe_scope_lifetime(),
        ))
    }

    async fn revoke_account_tokens(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Option<DateTime<Utc>>> {
        let mut state = self.account_revocations.lock().await;

        Ok(self.update_token_cutoff(
            &mut *state,
            self.account_revocations.clone(),
            account_id,
            TokenScope::safe_global_lifetime(),
        ))
    }

    async fn is_revoked(&self, token: &DecodedToken) -> RepoResult<bool> {
        Ok(self.check_fingerprint_revocation(token).await
            || self.check_scope_revocation(token).await
            || self.check_account_revocation(token).await)
    }
}

impl MockTokenRepoImpl {
    // Deadlock safety: Mutex gurad for the locked_state must be dropped before
    // any asynchronous call.
    fn update_token_cutoff<T: Hash + Eq + Send + Clone + 'static>(
        &self,
        locked_state: &mut HashMap<T, DateTime<Utc>>,
        state: Arc<Mutex<HashMap<T, DateTime<Utc>>>>,
        key: T,
        expiration: Duration,
    ) -> Option<DateTime<Utc>> {
        let cutoff_time = locked_state
            .entry(key.clone())
            .or_insert(DateTime::<Utc>::MIN_UTC);

        let previous_cutoff_time = *cutoff_time;

        *cutoff_time = std::cmp::max(checked_now(), *cutoff_time);
        let cutoff_time = *cutoff_time;

        // The clean up task removes revocation entry aftear any token subject
        // to cutoff has been expired.
        tokio::spawn(async move {
            tokio::time::sleep(expiration).await;
            let mut state = state.lock().await;

            if let Some(&current_cutoff_time) = state.get(&key)
                && current_cutoff_time == cutoff_time
            {
                state.remove(&key);
            }
        });

        if previous_cutoff_time == DateTime::<Utc>::MIN_UTC {
            None
        } else {
            Some(previous_cutoff_time)
        }
    }

    async fn check_fingerprint_revocation(&self, token: &DecodedToken) -> bool {
        self.fingerprint_revocations
            .lock()
            .await
            .contains(&token.fingerprint)
    }

    async fn check_scope_revocation(&self, token: &DecodedToken) -> bool {
        if let Some(&time) = self
            .scope_revocations
            .lock()
            .await
            .get(&(token.id, token.scope.scope_name()))
        {
            token.issued_at <= time
        } else {
            false
        }
    }

    async fn check_account_revocation(&self, token: &DecodedToken) -> bool {
        if let Some(&time) = self.account_revocations.lock().await.get(&token.id) {
            token.issued_at <= time
        } else {
            false
        }
    }
}
