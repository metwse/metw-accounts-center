use service::{
    dto::repo::EmailLimitingResult,
    repo::{EmailLimitingRepo, RepoResult},
    testutil::{random_email, random_ipv6},
};
use std::assert_matches;

/// Put limit to email addresses.
pub async fn email_limiting(repo: &dyn EmailLimitingRepo) -> RepoResult<()> {
    let ip = random_ipv6();
    let email = random_email();

    assert_matches!(
        repo.check_and_consume_quota(&ip, email).await?,
        EmailLimitingResult::Allowed
    );

    assert_matches!(
        repo.check_and_consume_quota(&ip, email).await?,
        EmailLimitingResult::EmailLimited(..) | EmailLimitingResult::IpLimited(..)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::email_limiting;
    use crate::util::redis_client_from_env;
    use service::repo::{EmailLimitingRepo, RepoResult, mock::MockEmailLimitingRepoImpl};
    use state::EmailLimitingRepoImpl;

    async fn testsuite(email_limiting_repo: &dyn EmailLimitingRepo) -> RepoResult<()> {
        email_limiting(email_limiting_repo).await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    async fn mock_email_limiting_repo() -> RepoResult<()> {
        testsuite(MockEmailLimitingRepoImpl::boxed_new().as_ref()).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    #[ignore]
    #[serial_test::serial]
    async fn email_limiting_repo() -> RepoResult<()> {
        let redis = redis_client_from_env().await;

        testsuite(EmailLimitingRepoImpl::boxed_new(redis.clone()).as_ref()).await
    }
}
