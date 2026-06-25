use crate::{
    checked_now,
    id::AccountId,
    repo::{
        EmailLimitingRepo, RepoResult, TokenRepo,
        mock::{MockEmailLimitingRepoImpl, MockTokenRepoImpl},
    },
    testutil::{random_email, random_ipv6, random_username},
    token::{DecodedToken, SAFE_EXPIRATION_MARGIN, TokenScope},
};
use std::{assert_matches, time::Duration};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn mock_token_repo_cleanup_task() -> RepoResult<()> {
    let repo = MockTokenRepoImpl::boxed_new();

    let token = DecodedToken {
        id: AccountId::unique(),
        scope: TokenScope::Session,
        expires_at: checked_now(),
        issued_at: checked_now(),
        fingerprint: random_username().into(),
    };

    assert!(!repo.check_and_revoke_token(&token).await?);

    // Now the token is revoked.
    assert!(repo.check_and_revoke_token(&token).await?);
    assert!(repo.is_revoked(&token).await?);

    tokio::time::advance(token.safe_lifetime() - SAFE_EXPIRATION_MARGIN).await;
    tokio::task::yield_now().await;

    // Token is still revoked.
    assert!(repo.check_and_revoke_token(&token).await?);

    tokio::time::advance(SAFE_EXPIRATION_MARGIN + Duration::from_secs(1)).await;
    tokio::task::yield_now().await;

    // Now it is not.
    assert!(!repo.is_revoked(&token).await?);

    // Same test, but for cutoff-cleanup tasks.
    assert!(!repo.check_and_revoke_account_tokens(&token).await?);
    assert!(repo.is_revoked(&token).await?);

    tokio::time::resume();
    tokio::time::sleep(Duration::from_millis(10)).await;

    repo.revoke_account_tokens(token.id).await?;

    tokio::time::sleep(Duration::from_millis(10)).await;
    tokio::time::pause();

    tokio::task::yield_now().await; // REG: To ensure the timer is registered.

    tokio::time::advance(TokenScope::safe_global_lifetime()).await;
    tokio::task::yield_now().await;

    assert!(!repo.is_revoked(&token).await?);

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn mock_email_limiting_repo_cleanup_task() -> RepoResult<()> {
    use crate::dto::repo::EmailLimitingResult;

    let repo = MockEmailLimitingRepoImpl::boxed_new();

    let ip = random_ipv6();
    let email = random_email();

    // Emulate sending 5 emails
    for _ in 0..5 {
        assert_matches!(
            repo.check_and_limit_email(&ip, email).await?,
            EmailLimitingResult::NoTimeOut
        );

        assert_matches!(
            repo.check_and_limit_email(&ip, email).await?,
            EmailLimitingResult::EmailTimeOut(..) | EmailLimitingResult::IpTimeOut(..)
        );

        tokio::task::yield_now().await; // REG: To ensure the timer is registered.

        tokio::time::advance(Duration::from_mins(1)).await;
        tokio::task::yield_now().await;
    }

    // Now the email's quota has been filled.
    let another_ip = random_ipv6();
    assert_matches!(
        repo.check_and_limit_email(&ip, email).await?,
        EmailLimitingResult::EmailTimeOut(..)
    );
    assert_matches!(
        repo.check_and_limit_email(&another_ip, email).await?,
        EmailLimitingResult::EmailTimeOut(..)
    );

    let another_email = random_email();
    // Send 5 more emails.
    for _ in 0..5 {
        assert_matches!(
            repo.check_and_limit_email(&ip, another_email).await?,
            EmailLimitingResult::NoTimeOut
        );

        tokio::task::yield_now().await; // REG.

        tokio::time::advance(Duration::from_mins(1)).await;
        tokio::task::yield_now().await;
    }

    // Now the IP's quota has been filled.
    let yet_another_email = random_email();
    assert_matches!(
        repo.check_and_limit_email(&ip, yet_another_email).await?,
        EmailLimitingResult::IpTimeOut(..)
    );

    tokio::task::yield_now().await; // REG.

    tokio::time::advance(Duration::from_hours(24)).await;
    tokio::task::yield_now().await;

    // Both the email's and IP's quota has been replenished.
    assert_matches!(
        repo.check_and_limit_email(&ip, email).await?,
        EmailLimitingResult::NoTimeOut
    );

    Ok(())
}
