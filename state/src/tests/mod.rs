mod accounts;
mod token_revocation;

use crate::{AccountRepoImpl, TokenRepoImpl};
use accounts::{account_creation, email_change};
use redis::aio::MultiplexedConnection;
use service::{
    repo::mock::{MockAccountRepoImpl, MockTokenRepoImpl},
    service::AccountService,
};
use sqlx::PgPool;
use std::sync::Arc;
use token_revocation::token_revocation;

async fn default_db() -> PgPool {
    dotenvy::dotenv().ok();

    PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}

async fn default_redis() -> MultiplexedConnection {
    dotenvy::dotenv().ok();

    redis::Client::open(std::env::var("REDIS_URL").unwrap())
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn mock_account_creation() {
    let account_service = Arc::new(AccountService::new(MockAccountRepoImpl::boxed_new()));

    account_creation(account_service.clone()).await.unwrap();
    let username1 = account_creation(account_service.clone()).await.unwrap();

    email_change(username1, account_service).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
#[ignore]
async fn db_account_creation() {
    let pool = default_db().await;

    let account_service = Arc::new(AccountService::new(AccountRepoImpl::boxed_new(
        pool.clone(),
    )));

    account_creation(account_service.clone()).await.unwrap();
    let username1 = account_creation(account_service.clone()).await.unwrap();

    email_change(username1, account_service).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn mock_token_revocation() {
    let token_repo = MockTokenRepoImpl::boxed_new();

    token_revocation(token_repo).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
#[ignore]
async fn redis_token_revocation() {
    let redis = default_redis().await;

    for _ in 0..16 {
        let token_repo = TokenRepoImpl::boxed_new(redis.clone());

        token_revocation(token_repo).await.unwrap();
    }
}
