mod accounts;
mod token_revocation;

use crate::{AccountRepoImpl, CaptchaClientImpl, Config, TokenRepoImpl};
use accounts::{account_creation, account_creation_data_race, email_change};
use token_revocation::{token_revocation, token_revocation_data_race};

use redis::aio::MultiplexedConnection;
use service::{
    repo::{
        RepoResult,
        mock::{MockAccountRepoImpl, MockTokenRepoImpl},
    },
    service::{AccountService, ServiceResult},
};
use sqlx::PgPool;
use std::sync::Arc;

async fn default_db() -> PgPool {
    dotenvy::dotenv_override().ok();

    PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}

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
async fn mock_account_creation() -> ServiceResult<()> {
    let account_service = Arc::new(AccountService::new(MockAccountRepoImpl::boxed_new()));

    account_creation_data_race(account_service.clone()).await?;

    account_creation(account_service.clone()).await?;
    let username1 = account_creation(account_service.clone()).await?;

    email_change(username1, account_service).await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
#[ignore]
#[serial_test::serial]
async fn db_account_creation() -> ServiceResult<()> {
    let pool = default_db().await;

    let account_service = Arc::new(AccountService::new(AccountRepoImpl::boxed_new(
        pool.clone(),
    )));

    account_creation_data_race(account_service.clone()).await?;

    account_creation(account_service.clone()).await?;
    let username1 = account_creation(account_service.clone()).await?;

    email_change(username1, account_service).await?;

    Ok(())
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

// Test validity of .env.example
#[test]
#[serial_test::serial]
fn config_from_example_env() {
    dotenvy::from_path_override(".env.example").unwrap();

    Config::from_env();
}

// Bootstrap all services and clients, for testing .env
#[tokio::test]
#[ignore]
#[serial_test::serial]
async fn state_from_env() {
    dotenvy::dotenv_override().unwrap();

    let config = Config::from_env();

    config.bootstrap().await;
}
