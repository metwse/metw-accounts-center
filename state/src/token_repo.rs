use async_trait::async_trait;
use chrono::{DateTime, Utc};
use redis::{AsyncCommands, aio::MultiplexedConnection};
use service::{
    checked_now,
    id::AccountId,
    repo::{RepoResult, TokenRepo},
    token::{DecodedToken, TokenScope},
};
use std::time::Duration;

/// Token repository using Redis.
pub struct TokenRepoImpl {
    con: MultiplexedConnection,
}

impl TokenRepoImpl {
    /// Creates a new token repository.
    pub fn boxed_new(con: MultiplexedConnection) -> Box<Self> {
        Box::new(Self { con })
    }
}

#[async_trait]
impl TokenRepo for TokenRepoImpl {
    async fn check_and_revoke_token(&self, token: &DecodedToken) -> RepoResult<bool> {
        let (is_scope_revoked, is_account_revoked) = tokio::try_join!(
            self.check_scope_revocation(token),
            self.check_account_revocation(token)
        )?;

        if is_scope_revoked || is_account_revoked {
            return Ok(true);
        }

        let mut con = self.con.clone();

        Ok(!con
            .set_options::<'_, Vec<u8>, &str, bool>(
                to_fingerprint_key(token),
                "",
                redis::SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::PX(
                        token.safe_lifetime().as_millis() as u64
                    )),
            )
            .await?)
    }

    async fn check_and_revoke_account_tokens_with_scope(
        &self,
        token: &DecodedToken,
    ) -> RepoResult<bool> {
        let (is_fingerprint_revoked, is_account_revoked) = tokio::try_join!(
            self.check_fingerprint_revocation(token),
            self.check_account_revocation(token)
        )?;

        if is_fingerprint_revoked || is_account_revoked {
            return Ok(true);
        }

        let previous_token_cutoff_time = self
            .revoke_account_tokens_with_scope(token.id, &token.scope)
            .await?;

        if let Some(previous_token_cutoff_time) = previous_token_cutoff_time {
            Ok(token.issued_at <= previous_token_cutoff_time)
        } else {
            Ok(false)
        }
    }

    async fn check_and_revoke_account_tokens(&self, token: &DecodedToken) -> RepoResult<bool> {
        let (is_fingerprint_revoked, is_scope_revoked) = tokio::try_join!(
            self.check_fingerprint_revocation(token),
            self.check_scope_revocation(token)
        )?;

        if is_fingerprint_revoked || is_scope_revoked {
            return Ok(true);
        }

        let previous_token_cutoff_time = self.revoke_account_tokens(token.id).await?;

        if let Some(previous_token_cutoff_time) = previous_token_cutoff_time {
            Ok(token.issued_at <= previous_token_cutoff_time)
        } else {
            Ok(false)
        }
    }

    async fn revoke_account_tokens_with_scope(
        &self,
        account_id: AccountId,
        scope: &TokenScope,
    ) -> RepoResult<Option<DateTime<Utc>>> {
        let key = to_scope_key(account_id, scope);

        self.update_token_cutoff_time(key, scope.safe_scope_lifetime())
            .await
    }

    async fn revoke_account_tokens(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Option<DateTime<Utc>>> {
        let key = to_account_key(account_id);

        self.update_token_cutoff_time(key, TokenScope::safe_global_lifetime())
            .await
    }

    async fn is_revoked(&self, token: &DecodedToken) -> RepoResult<bool> {
        let (is_fingerprint_revoked, is_scope_revoked, is_account_revoked) = tokio::try_join!(
            self.check_fingerprint_revocation(token),
            self.check_scope_revocation(token),
            self.check_account_revocation(token)
        )?;

        Ok(is_fingerprint_revoked || is_scope_revoked || is_account_revoked)
    }
}

impl TokenRepoImpl {
    // Update token cutoff time to now, and return previous cutoff it exists.
    // The cutoff with hold for "expiration". Provide a safe duration so that
    // any token affected by the revocation will expire while until cutoff is
    // held.
    async fn update_token_cutoff_time(
        &self,
        key: String,
        expiration: Duration,
    ) -> RepoResult<Option<DateTime<Utc>>> {
        let con = self.con.clone();

        let previous_token_cutoff_time: (Option<i64>,) =
            redis::aio::transaction_async(con, &[&key], |mut con, mut pipe| {
                let now = checked_now().timestamp_millis();
                let key = key.clone();

                async move {
                    let existing: Option<i64> = con.get(&key).await?;

                    let cutoff_time = match existing {
                        Some(v) => v.max(now),
                        None => now,
                    };

                    pipe.set_options(
                        &key,
                        cutoff_time,
                        redis::SetOptions::default()
                            .with_expiration(redis::SetExpiry::PX(expiration.as_millis() as u64))
                            .get(true),
                    )
                    .query_async(&mut con)
                    .await
                }
            })
            .await?;

        Ok(previous_token_cutoff_time
            .0
            .map(|timestamp_millis| DateTime::from_timestamp_millis(timestamp_millis).unwrap()))
    }

    async fn check_fingerprint_revocation(&self, token: &DecodedToken) -> RepoResult<bool> {
        Ok(self
            .con
            .clone()
            .exists::<'_, Vec<u8>, bool>(to_fingerprint_key(token))
            .await?)
    }

    async fn check_scope_revocation(&self, token: &DecodedToken) -> RepoResult<bool> {
        let token_cutoff_time = self
            .con
            .clone()
            .get::<'_, String, Option<i64>>(to_scope_key(token.id, &token.scope))
            .await?;

        if let Some(token_cutoff_time) = token_cutoff_time {
            Ok(token.issued_at.timestamp_millis() <= token_cutoff_time)
        } else {
            Ok(false)
        }
    }

    async fn check_account_revocation(&self, token: &DecodedToken) -> RepoResult<bool> {
        let token_cutoff_time = self
            .con
            .clone()
            .get::<'_, String, Option<i64>>(to_account_key(token.id))
            .await?;

        if let Some(token_cutoff_time) = token_cutoff_time {
            Ok(token.issued_at.timestamp_millis() <= token_cutoff_time)
        } else {
            Ok(false)
        }
    }
}

fn to_fingerprint_key(token: &DecodedToken) -> Vec<u8> {
    let mut key = b"revoke-token:".to_vec();
    key.extend(&token.fingerprint);
    key
}

fn to_scope_key(account_id: AccountId, scope: &TokenScope) -> String {
    format!("revoke-token:scope:{}:{}", account_id, scope.scope_name())
}

fn to_account_key(account_id: AccountId) -> String {
    format!("revoke-token:account:{}", account_id)
}
