use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use service::{
    checked_now,
    repo::{RepoResult, TokenRepo},
    token::{DecodedToken, TokenScope},
};

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
    async fn revoke_fingerprint(&self, token: &DecodedToken) -> RepoResult<bool> {
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

    async fn revoke_scope(&self, token: &DecodedToken) -> RepoResult<bool> {
        let (is_fingerprint_revoked, is_account_revoked) = tokio::try_join!(
            self.check_fingerprint_revocation(token),
            self.check_account_revocation(token)
        )?;

        if is_fingerprint_revoked || is_account_revoked {
            return Ok(true);
        }

        let con = self.con.clone();
        let key = to_scope_key(token);

        let token_cutoff: (Option<i64>,) =
            redis::aio::transaction_async(con, &[&key], |mut con, mut pipe| {
                let now = checked_now().timestamp_millis();
                let key = key.clone();

                async move {
                    let existing: Option<i64> = con.get(&key).await?;

                    let revoke_before = match existing {
                        Some(v) => v.max(now),
                        None => now,
                    };

                    pipe.set_options(
                        &key,
                        revoke_before,
                        redis::SetOptions::default()
                            .with_expiration(redis::SetExpiry::PX(
                                token.scope.safe_scope_lifetime().as_millis() as u64,
                            ))
                            .get(true),
                    )
                    .query_async(&mut con)
                    .await
                }
            })
            .await?;

        if let Some(token_cutoff) = token_cutoff.0 {
            Ok(token.issued_at.timestamp_millis() <= token_cutoff)
        } else {
            Ok(false)
        }
    }

    async fn revoke_account(&self, token: &DecodedToken) -> RepoResult<bool> {
        let (is_fingerprint_revoked, is_scope_revoked) = tokio::try_join!(
            self.check_fingerprint_revocation(token),
            self.check_scope_revocation(token)
        )?;

        if is_fingerprint_revoked || is_scope_revoked {
            return Ok(true);
        }

        let con = self.con.clone();
        let key = to_account_key(token);

        let token_cutoff: (Option<i64>,) =
            redis::aio::transaction_async(con, &[&key], |mut con, mut pipe| {
                let now = checked_now().timestamp_millis();
                let key = key.clone();

                async move {
                    let existing: Option<i64> = con.get(&key).await?;

                    let revoke_before = match existing {
                        Some(v) => v.max(now),
                        None => now,
                    };

                    pipe.set_options(
                        &key,
                        revoke_before,
                        redis::SetOptions::default()
                            .with_expiration(redis::SetExpiry::PX(
                                TokenScope::safe_global_lifetime().as_millis() as u64,
                            ))
                            .get(true),
                    )
                    .query_async(&mut con)
                    .await
                }
            })
            .await?;

        if let Some(token_cutoff) = token_cutoff.0 {
            Ok(token.issued_at.timestamp_millis() <= token_cutoff)
        } else {
            Ok(false)
        }
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
    async fn check_fingerprint_revocation(&self, token: &DecodedToken) -> RepoResult<bool> {
        Ok(self
            .con
            .clone()
            .exists::<'_, Vec<u8>, bool>(to_fingerprint_key(token))
            .await?)
    }

    async fn check_scope_revocation(&self, token: &DecodedToken) -> RepoResult<bool> {
        let token_cutoff = self
            .con
            .clone()
            .get::<'_, String, Option<i64>>(to_scope_key(token))
            .await?;

        if let Some(token_cutoff) = token_cutoff {
            Ok(token.issued_at.timestamp_millis() <= token_cutoff)
        } else {
            Ok(false)
        }
    }

    async fn check_account_revocation(&self, token: &DecodedToken) -> RepoResult<bool> {
        let token_cutoff = self
            .con
            .clone()
            .get::<'_, String, Option<i64>>(to_account_key(token))
            .await?;

        if let Some(token_cutoff) = token_cutoff {
            Ok(token.issued_at.timestamp_millis() <= token_cutoff)
        } else {
            Ok(false)
        }
    }
}

fn to_fingerprint_key(token: &DecodedToken) -> Vec<u8> {
    let mut key = b"revoke-token:fingerprint:".to_vec();
    key.extend(&token.fingerprint);
    key
}

fn to_scope_key(token: &DecodedToken) -> String {
    format!(
        "revoke-token:scope:{}:{}",
        token.scope.variant_name(),
        token.id
    )
}

fn to_account_key(token: &DecodedToken) -> String {
    format!("revoke-token:account:{}", token.id)
}
