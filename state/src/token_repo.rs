use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use service::repo::{RepoResult, TokenRepo};

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
    async fn check_and_revoke(
        &self,
        fingerprint: &[u8],
        revoke_for: std::time::Duration,
    ) -> RepoResult<bool> {
        let mut con = self.con.clone();

        Ok(!con
            .set_options::<'_, &[u8], &str, bool>(
                fingerprint,
                "",
                redis::SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::PX(revoke_for.as_millis() as u64)),
            )
            .await?)
    }

    async fn check_revocation(&self, fingerprint: &[u8]) -> RepoResult<bool> {
        let mut con = self.con.clone();

        Ok(con.exists::<'_, &[u8], bool>(fingerprint).await?)
    }
}
