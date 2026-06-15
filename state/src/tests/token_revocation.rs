use std::time;

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
