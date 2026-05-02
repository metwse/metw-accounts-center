use super::*;
use crate::repo::mock::new_mock;

#[tokio::test(flavor = "multi_thread")]
async fn account_creation_mock_mt() -> ServiceResult<()> {
    let mock_repo = new_mock();

    let account_service = AccountService::new(mock_repo);

    let mut signup_dto = dto::request::Signup {
        username: "test".to_string(),
        email: "me [at] metehanselvi [dot] com".to_string(),
        password_hash: "".to_string(),
        keys: dto::request::Keys {
            identity_key: vec![],
            encrypted_private_key: vec![],
            encrypted_master_key: vec![],
        },
    };

    account_service.signup(signup_dto.clone()).await?;

    signup_dto.email += "test";
    signup_dto.username += "test";

    account_service.signup(signup_dto).await?;

    Ok(())
}
