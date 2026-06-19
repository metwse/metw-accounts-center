use std::time::{self, Duration};

use service::{
    repo::{RepoResult, TokenRepo},
    testutil::random_username,
};

/// Sign a token, then check and revoke.
pub async fn token_revocation(repo: Box<dyn TokenRepo>) -> RepoResult<()> {
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
pub async fn token_revocation_data_race(repo: Box<dyn TokenRepo>) -> RepoResult<()> {
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
    use state::{CaptchaClientImpl, TokenRepoImpl};

    use redis::aio::MultiplexedConnection;
    use service::repo::{RepoResult, mock::MockTokenRepoImpl};

    async fn default_redis() -> MultiplexedConnection {
        dotenvy::dotenv_override().ok();

        redis::Client::open(std::env::var("REDIS_URL").unwrap())
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    async fn mock_token_revocation() -> RepoResult<()> {
        token_revocation(MockTokenRepoImpl::boxed_new()).await?;

        token_revocation_data_race(MockTokenRepoImpl::boxed_new()).await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    #[ignore]
    #[serial_test::serial]
    async fn redis_token_revocation() -> RepoResult<()> {
        let redis = default_redis().await;

        for _ in 0..16 {
            token_revocation(TokenRepoImpl::boxed_new(redis.clone())).await?;

            token_revocation_data_race(TokenRepoImpl::boxed_new(redis.clone())).await?;
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn cloudflare_captcha() {
        const ALWAYS_PASS: &str = "1x0000000000000000000000000000000AA";
        const ALWAYS_FAIL: &str = "2x0000000000000000000000000000000AA";
        const ALWAYS_FAIL_ALREADY_SPENT: &str = "3x0000000000000000000000000000000AA";

        assert!(
            CaptchaClientImpl::boxed_new(ALWAYS_PASS.into())
                .validate("123".into())
                .await
        );
        assert!(
            !CaptchaClientImpl::boxed_new(ALWAYS_FAIL.into())
                .validate("123".into())
                .await
        );
        assert!(
            !CaptchaClientImpl::boxed_new(ALWAYS_FAIL_ALREADY_SPENT.into())
                .validate("123".into())
                .await
        );
    }
}
