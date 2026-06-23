use service::{
    checked_now,
    id::AccountId,
    repo::{RepoResult, TokenRepo},
    testutil::random_username,
    token::{DecodedToken, TokenScope},
};
use std::time::Duration;

/// Sign a token, then check and revoke.
pub async fn token_revocation(repo: &dyn TokenRepo) -> RepoResult<()> {
    let mut token = DecodedToken {
        id: AccountId::unique(),
        scope: service::token::TokenScope::Session,
        fingerprint: random_username().into(),
        expires_at: checked_now() + Duration::from_secs(1),
        issued_at: checked_now() - Duration::from_secs(1),
    };

    // --- Revoke scope ---
    assert!(!repo.is_revoked(&token).await?);

    // Revoke the token fingerprint.
    assert!(!repo.check_and_revoke_token(&token).await?);
    assert!(repo.is_revoked(&token).await?);

    // Change the fingerprint. Now the new token is not revoked.
    token.fingerprint = random_username().into();
    assert!(!repo.is_revoked(&token).await?);

    // Revoke tokens with Session scope.
    assert!(
        !repo
            .check_and_revoke_account_tokens_with_scope(&token)
            .await?
    );
    assert!(repo.is_revoked(&token).await?);

    // Tokens with different fingerprints also revoked.
    token.fingerprint = random_username().into();
    assert!(repo.is_revoked(&token).await?);

    // Newer tokens should not be revoked.
    token.issued_at += Duration::from_millis(1500);
    assert!(!repo.is_revoked(&token).await?);
    token.issued_at -= Duration::from_millis(1500);

    // But older one are revoced.
    token.issued_at -= Duration::from_millis(1500);
    assert!(repo.is_revoked(&token).await?);
    token.issued_at += Duration::from_millis(1500);

    // Retrying revocation should fail.
    assert!(
        repo.check_and_revoke_account_tokens_with_scope(&token)
            .await?
    );

    // Cannot revoke fingerprint or account as the scope is revoked.
    assert!(repo.check_and_revoke_token(&token).await?);
    assert!(repo.check_and_revoke_account_tokens(&token).await?);

    token.scope = service::token::TokenScope::EmailVerificationSession;

    // But other scopes are still valid.
    assert!(!repo.is_revoked(&token).await?);
    token.scope = service::token::TokenScope::Session;

    // --- Revoke account ---
    token.id = AccountId::unique();
    token.fingerprint = random_username().into();

    assert!(!repo.is_revoked(&token).await?);

    // Revoke all tokens of the account.
    assert!(!repo.check_and_revoke_account_tokens(&token).await?);

    // Newer tokens should not be revoked.
    token.issued_at += Duration::from_millis(1500);
    assert!(!repo.is_revoked(&token).await?);
    token.issued_at -= Duration::from_millis(1500);

    // But older ones are revoked.
    token.issued_at -= Duration::from_millis(1500);
    assert!(repo.is_revoked(&token).await?);
    token.issued_at += Duration::from_millis(1500);

    // Retrying revocation should fail.
    assert!(repo.check_and_revoke_account_tokens(&token).await?);
    assert!(repo.check_and_revoke_token(&token).await?); // should not revoke fingerprint
    assert!(
        repo.check_and_revoke_account_tokens_with_scope(&token)
            .await?
    );

    token.id = AccountId::unique();

    // Fingerprint was vaild until this point.
    assert!(!repo.check_and_revoke_token(&token).await?);
    assert!(repo.check_and_revoke_token(&token).await?);

    Ok(())
}

/// Concurrently call `check_and_revoke`. Only one of the requests should
/// return `false`.
pub async fn token_revocation_data_race(repo: &dyn TokenRepo) -> RepoResult<()> {
    let mut token_revocation_futures = Vec::with_capacity(16);

    let token = DecodedToken {
        id: AccountId::unique(),
        scope: TokenScope::Session,
        fingerprint: random_username().into(),
        expires_at: checked_now(),
        issued_at: checked_now(),
    };

    for _ in 0..16 {
        token_revocation_futures.push({
            let token = token.clone();
            async move { repo.check_and_revoke_token(&token).await }
        });
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
