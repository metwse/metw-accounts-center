mod account_creation;
mod token_revocation;

use account_creation::account_creation;
use token_revocation::token_revocation;

use crate::AccountRepoImpl;
use service::repo::mock::{MockAccountRepoImpl, MockTokenRepoImpl};
use sqlx::PgPool;

async fn default_db() -> PgPool {
    dotenvy::dotenv().ok();

    PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn mock_account_creation() {
    let account_repo = MockAccountRepoImpl::boxed_new();

    account_creation(account_repo).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
#[ignore]
async fn db_account_creation() {
    let pool = default_db().await;

    let account_repo = AccountRepoImpl::boxed_new(pool);

    account_creation(account_repo).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn mock_token_revocation() {
    let token_repo = MockTokenRepoImpl::boxed_new();

    token_revocation(token_repo).await.unwrap();
}
