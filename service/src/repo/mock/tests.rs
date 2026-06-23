use crate::{
    checked_now,
    id::AccountId,
    repo::{RepoResult, TokenRepo, mock::MockTokenRepoImpl},
    testutil::random_username,
    token::{DecodedToken, SAFE_EXPIRATION_MARGIN, TokenScope},
};
use std::time::Duration;

#[tokio::test(flavor = "current_thread")]
async fn cleanup_task() -> RepoResult<()> {
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

    tokio::time::pause();
    tokio::time::advance(token.safe_lifetime() - SAFE_EXPIRATION_MARGIN).await;
    tokio::time::resume();

    // Token is still revoked.
    assert!(repo.check_and_revoke_token(&token).await?);

    tokio::time::pause();
    tokio::time::advance(SAFE_EXPIRATION_MARGIN + Duration::from_secs(1)).await;
    tokio::time::resume();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Now it is not.
    assert!(!repo.is_revoked(&token).await?);

    Ok(())
}
