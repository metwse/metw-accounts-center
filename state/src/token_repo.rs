use async_trait::async_trait;
use service::repo::{RepoResult, TokenRepo};

/// Token repository using Redis.
pub struct TokenRepoImpl;

#[async_trait]
impl TokenRepo for TokenRepoImpl {
    async fn check_and_revoke(
        &self,
        _fingerprint: &[u8],
        _revoke_for: std::time::Duration,
    ) -> RepoResult<bool> {
        todo!()
    }

    async fn check_revocation(&self, _fingerprint: &[u8]) -> RepoResult<bool> {
        todo!()
    }
}
