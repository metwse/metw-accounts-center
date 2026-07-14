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

    // Now the IP can send another email. But the address is still blocked
    repo.refund_ip_quota(&ip, email).await?;
    let another_email = random_email();

    assert_matches!(
        repo.check_and_consume_quota(&ip, email).await?,
        EmailLimitingResult::EmailLimited(..)
    );

    assert_matches!(
        repo.check_and_consume_quota(&ip, another_email).await?,
        EmailLimitingResult::Allowed
    );

    // Clear the limits on `another_email`
    repo.clear_email_limit(another_email).await?;
    let another_ip = random_ipv6();

    assert_matches!(
        repo.check_and_consume_quota(&ip, another_email).await?,
        EmailLimitingResult::IpLimited(..)
    );

    assert_matches!(
        repo.check_and_consume_quota(&another_ip, another_email)
            .await?,
        EmailLimitingResult::Allowed
    );

    // This does not unblock the IP, as it is blocked due to `email`
    repo.refund_ip_quota(&another_ip, email).await?;
    let yet_another_email = random_email();

    assert_matches!(
        repo.check_and_consume_quota(&ip, yet_another_email).await?,
        EmailLimitingResult::IpLimited(..)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::email_limiting;
    use crate::util::redis_con_generator_from_env;
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
        let con_generator = redis_con_generator_from_env().await;

        testsuite(
            EmailLimitingRepoImpl::boxed_new(&con_generator)
                .await
                .as_ref(),
        )
        .await
    }
}
