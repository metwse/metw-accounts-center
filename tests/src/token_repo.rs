use service::{
    repo::{RepoResult, TokenRepo},
    testutil::random_username,
};
use std::time::{self, Duration};

/// Sign a token, then check and revoke.
pub async fn token_revocation(repo: &dyn TokenRepo) -> RepoResult<()> {
    // Let's use snowflake id as random fingerprint.
    let random_fingerprint = random_username();
    let another_random_fingerprint = random_username();

    // The "fingerprint", random string, has never revoked.
    assert!(!repo.check_revocation(random_fingerprint.as_bytes()).await?);
    assert!(
        !repo
            .check_revocation(another_random_fingerprint.as_bytes())
            .await?
    );

    // Now the fingerprint is revoked.
    assert!(
        !repo
            .check_and_revoke(
                random_fingerprint.as_bytes(),
                time::Duration::from_secs(100)
            )
            .await?
    );

    // Revoke should return true.
    assert!(repo.check_revocation(random_fingerprint.as_bytes()).await?);
    // The other fingerprint should stay the valid.
    assert!(
        !repo
            .check_revocation(another_random_fingerprint.as_bytes())
            .await?
    );

    Ok(())
}

/// Concurrently call `check_and_revoke`. Only one of the requests should
/// return `false`.
pub async fn token_revocation_data_race(repo: &dyn TokenRepo) -> RepoResult<()> {
    let random_fingerprint = random_username();

    let mut token_revocation_futures = Vec::with_capacity(16);

    for _ in 0..16 {
        token_revocation_futures
            .push(repo.check_and_revoke(random_fingerprint.as_bytes(), Duration::from_mins(1)));
    }

    let token_revocation_results = futures_util::future::join_all(token_revocation_futures).await;

    // Accept the token only once.
    assert!(
        token_revocation_results
            .iter()
            .filter(|is_revoked| !is_revoked.as_ref().unwrap())
            .count()
            == 1
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{token_revocation, token_revocation_data_race};
    use crate::util::redis_client_from_env;
    use service::repo::{RepoResult, TokenRepo, mock::MockTokenRepoImpl};
    use state::TokenRepoImpl;

    async fn testsuite(token_repo: &dyn TokenRepo) -> RepoResult<()> {
        for _ in 0..4 {
            token_revocation(token_repo).await?;

            token_revocation_data_race(token_repo).await?;
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    async fn mock_token_repo() -> RepoResult<()> {
        testsuite(MockTokenRepoImpl::boxed_new().as_ref()).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    #[ignore]
    #[serial_test::serial]
    async fn token_repo() -> RepoResult<()> {
        let redis = redis_client_from_env().await;

        testsuite(TokenRepoImpl::boxed_new(redis.clone()).as_ref()).await
    }
}
