use super::{AccountService, ServiceResult};
use crate::{
    dto, entity,
    repo::impls::{MockAccountRepoImpl, MockTokenRepoImpl},
    service::{ServiceError, TokenService},
    token::{Token, TokenScope},
    util::check_password,
};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
async fn account_creation_mock_mt() -> ServiceResult<()> {
    let repo = MockAccountRepoImpl::boxed_new();

    let account_service = AccountService::new(repo);

    let mut signup_dto = dto::request::Signup {
        username: "user1".to_string(),
        email: "user1@example.com".to_string(),
        password_hash: "paswd1".to_string(),
        keys: dto::request::Keys {
            identity_key: vec![1],
            encrypted_private_key: vec![2],
            encrypted_master_key: vec![3],
        },
    };

    let user1_account_id = account_service.signup(signup_dto.clone()).await?;

    signup_dto.username = "user2".to_string();
    signup_dto.email = "user2@example.com".to_string();
    signup_dto.password_hash = "paswd2".to_string();

    let user2_account_id = account_service.signup(signup_dto).await?;

    let user2_account_id_login = account_service
        .login_with_username(dto::request::LoginWithUsername {
            username: "user2".to_string(),
            password_hash: "paswd2".to_string(),
        })
        .await?;

    let repo = account_service.repo;

    assert!(user2_account_id_login == user2_account_id);

    assert!(repo.get_primary_username(user1_account_id).await?.unwrap() == "user1");
    assert!(
        check_password(
            "paswd1".to_string(),
            repo.get_login_by_username("user1")
                .await?
                .unwrap()
                .password_hash
        )
        .await
    );
    assert!(
        repo.get_login_by_email("user1@example.com")
            .await?
            .is_none()
    );

    assert!(repo.get_primary_username(user2_account_id).await?.unwrap() == "user2");
    assert!(
        repo.get_keys(user2_account_id).await?.unwrap()
            == dto::repo::Keys {
                identity_key: vec![1],
                encrypted_private_key: vec![2],
                encrypted_master_key: vec![3],
            }
    );

    Ok(())
}

#[tokio::test]
async fn token_service() -> ServiceResult<()> {
    let repo = MockTokenRepoImpl::boxed_new();

    let token_service = TokenService::new(repo, b"supersecret1234");

    let token1 = Token::new(
        entity::AccountId(0),
        vec![TokenScope::Authenticate],
        Duration::from_secs(1000),
    );

    let token2 = Token::new(
        entity::AccountId(0),
        vec![TokenScope::Authenticate],
        Duration::from_secs(0),
    );

    let token3 = Token::new(
        entity::AccountId(0),
        vec![TokenScope::Authenticate],
        Duration::from_secs(101),
    );

    let signed1 = token_service.sign(&token1);
    let signed2 = token_service.sign(&token2);
    let signed3 = token_service.sign(&token3);

    token_service.verify(&signed1).await?;

    token_service.revoke(&signed1).await?;

    assert!(matches!(
        token_service.verify(&signed1).await,
        Err(ServiceError::TokenRevoked)
    ));

    assert!(matches!(
        token_service.revoke(&signed1).await,
        Err(ServiceError::TokenRevoked)
    ));

    token_service.verify(&signed2).await?;
    token_service.revoke(&signed2).await?;

    token_service.verify(&signed3).await?;

    for invalid_jwt in ["invalid", "", "invalid.invalid", "invalid.invalid.invalid"] {
        assert!(matches!(
            token_service.revoke(invalid_jwt).await,
            Err(ServiceError::InvalidJwt)
        ));
    }

    Ok(())
}
