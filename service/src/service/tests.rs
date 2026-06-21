use super::ServiceResult;
use crate::{
    id::AccountId,
    repo::mock::MockTokenRepoImpl,
    service::{ServiceError, TokenService},
    token::{Token, TokenScope},
};
use chrono::TimeDelta;
use futures_util::future::join_all;
use std::{assert_matches, sync::Arc};

#[tokio::test]
#[test_log::test]
async fn token_service_expired() -> ServiceResult<()> {
    let repo = MockTokenRepoImpl::boxed_new();

    let token_service = TokenService::new(repo, "supersecret1234".into());

    let token = Token {
        id: AccountId::unique(),
        scope: TokenScope::Session,
    };

    token_service
        .add_time_delta(TimeDelta::from_std(TokenScope::Session.safe_scope_lifetime()).unwrap());

    let signed = token_service.sign(&token);

    // Expire token4
    token_service.add_time_delta(TimeDelta::minutes(-1));

    assert_matches!(
        token_service.verify(&signed).await,
        Err(ServiceError::InvalidJwt)
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn token_service_toctou() -> ServiceResult<()> {
    let repo = MockTokenRepoImpl::boxed_new();

    let token_service = Arc::new(TokenService::new(repo, "supersecret1234".into()));

    let token = Token {
        id: AccountId::unique(),
        scope: TokenScope::Session,
    };

    let signed = token_service.sign(&token);

    let mut futures = Vec::with_capacity(32);

    // Spawn 32 concurrent futures.
    for _ in 0..32 {
        let token_service = Arc::clone(&token_service);
        let signed = signed.clone();

        futures.push(async move { token_service.revoke(&signed).await });
    }

    let results = join_all(futures).await;

    // Only one of the futures can be succeed.
    let total_success = results.iter().filter(|res| res.is_ok()).count();

    assert!(total_success == 1);

    Ok(())
}
