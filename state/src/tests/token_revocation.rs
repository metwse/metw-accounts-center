use std::time::{self, Duration};

use service::{
    repo::{RepoResult, TokenRepo},
    testutil::random_username,
};

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
