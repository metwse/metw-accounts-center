mod account_creation;
mod token_revocation;

use account_creation::account_creation;
use token_revocation::token_revocation;

use service::repo::mock::{MockAccountRepoImpl, MockTokenRepoImpl};

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn mock_account_creation() {
    let account_repo = MockAccountRepoImpl::boxed_new();

    account_creation(account_repo).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn mock_token_revocation() {
    let token_repo = MockTokenRepoImpl::boxed_new();

    token_revocation(token_repo).await.unwrap();
}
