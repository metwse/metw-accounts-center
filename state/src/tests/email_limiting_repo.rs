use crate::{EmailLimitingRepoImpl, redis_keys::email_limiting_repo::*};
use redis::AsyncCommands;
use service::{
    dto::repo::EmailLimitingResult,
    repo::{EmailLimitingRepo, RepoResult},
    testutil::{random_email, random_ipv6},
};
use std::assert_matches;
use tests::util::redis_con_generator_from_env;

#[tokio::test]
#[test_log::test]
#[ignore]
#[serial_test::serial]
async fn email_limiting_repo() -> RepoResult<()> {
    let con_generator = redis_con_generator_from_env().await;
    let repo = EmailLimitingRepoImpl::boxed_new(&con_generator).await;

    let mut con = con_generator().await;

    let ip = random_ipv6();
    let email = random_email();

    // Emulate sending 5 emails
    for _ in 0..5 {
        assert_matches!(
            repo.check_and_consume_quota(&ip, email).await?,
            EmailLimitingResult::Allowed
        );

        assert_matches!(
            repo.check_and_consume_quota(&ip, email).await?,
            EmailLimitingResult::EmailLimited(..) | EmailLimitingResult::IpLimited(..)
        );

        con.del::<String, u64>(to_block_ip_key(&ip)).await?;
        con.del::<String, u64>(to_block_email_key(email)).await?;
    }

    // Now the email's quota has been filled.
    let another_ip = random_ipv6();
    assert_matches!(
        repo.check_and_consume_quota(&ip, email).await?,
        EmailLimitingResult::EmailLimited(..)
    );
    assert_matches!(
        repo.check_and_consume_quota(&another_ip, email).await?,
        EmailLimitingResult::EmailLimited(..)
    );

    let another_email = random_email();
    // Send 5 more emails.
    for _ in 0..5 {
        assert_matches!(
            repo.check_and_consume_quota(&ip, another_email).await?,
            EmailLimitingResult::Allowed
        );

        con.del::<String, u64>(to_block_ip_key(&ip)).await?;
        con.del::<String, u64>(to_block_email_key(another_email))
            .await?;
    }

    // Now the IP's quota has been filled.
    let yet_another_email = random_email();
    assert_matches!(
        repo.check_and_consume_quota(&ip, yet_another_email).await?,
        EmailLimitingResult::IpLimited(..)
    );

    con.del::<String, u64>(to_block_ip_key(&ip)).await?;
    con.del::<String, u64>(to_block_email_key(email)).await?;
    con.del::<String, u64>(to_used_ip_quota_key(&ip)).await?;
    con.del::<String, u64>(to_used_email_quota_key(email))
        .await?;

    // Both the email's and IP's quota has been replenished.
    assert_matches!(
        repo.check_and_consume_quota(&ip, email).await?,
        EmailLimitingResult::Allowed
    );

    Ok(())
}
