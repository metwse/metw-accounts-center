use super::{AccountService, ServiceResult};
use crate::{
    dto, entity,
    repo::impls::{MockAccountRepoImpl, MockTokenRepoImpl},
    service::{ServiceError, TokenService},
    token::{Token, TokenScope},
    util::check_password,
};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")] // multi_thread used to test Send+Sync
async fn account_creation_mock_mt() -> ServiceResult<()> {
    let repo = MockAccountRepoImpl::boxed_new();

    let account_service = AccountService::new(repo);

    // This basic sign up request will be used for all accounts.
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

    let user2_account_id = account_service.signup(signup_dto.clone()).await?;

    // Try to register an account with already-taken username.
    assert!(matches!(
        account_service.signup(signup_dto).await,
        Err(ServiceError::UsernameTaken)
    ));

    // Log into user2 account.
    let user2_account_id_login = account_service
        .login_with_username(dto::request::LoginWithUsername {
            username: "user2".to_string(),
            password_hash: "paswd2".to_string(),
        })
        .await?;
    // Is id returned from login same with sign up?
    assert!(user2_account_id_login == user2_account_id);

    // Try logging with invalid credentials.
    assert!(matches!(
        account_service
            .login_with_username(dto::request::LoginWithUsername {
                username: "invalid_username".to_string(),
                password_hash: "paswd2".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    ));
    assert!(matches!(
        account_service
            .login_with_username(dto::request::LoginWithUsername {
                username: "user2".to_string(),
                password_hash: "invalid_password".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    ));

    // Get /me.
    account_service.me(user2_account_id).await?;

    // Get /me from non-existent account.
    assert!(matches!(
        account_service.me(entity::AccountId(0)).await,
        Err(ServiceError::AccountNotFound)
    ));

    // Consume the account_service, dive into `repo` layer.
    let repo = account_service.repo;

    // Validate account creation.
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
    // Email should not be added as we did not verified it.
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

    // Add email to the account, and elaborate abstraction to account_service
    // again.
    let mut transaction = repo.begin_transaction().await?;
    transaction
        .add_email(user1_account_id, "user1@example.com")
        .await?;
    transaction
        .set_primary_email(user1_account_id, "user1@example.com", true)
        .await?;
    transaction.commit().await?;

    let account_service = AccountService::new(repo);

    // Log into user1 account.
    let user1_account_id_login = account_service
        .login_with_email(dto::request::LoginWithEmail {
            email: "user1@example.com".to_string(),
            password_hash: "paswd1".to_string(),
        })
        .await?;
    // Is id returned from login same with sign up?
    assert!(user1_account_id_login == user1_account_id);

    // Also try invalid emails.
    assert!(matches!(
        account_service
            .login_with_email(dto::request::LoginWithEmail {
                email: "invalid_email".to_string(),
                password_hash: "paswd1".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    ));
    assert!(matches!(
        account_service
            .login_with_email(dto::request::LoginWithEmail {
                email: "user1@example.com".to_string(),
                password_hash: "invalid_password".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    ));

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

    let signed1 = token_service.sign(&token1);
    let signed2 = token_service.sign(&token2);

    // token1 should be valid.
    token_service.verify(&signed1).await?;

    // Revoke token1 and check revocation status.
    token_service.revoke(&signed1).await?;
    assert!(matches!(
        token_service.verify(&signed1).await,
        Err(ServiceError::TokenRevoked)
    ));
    // Revocation of already-revoked token returns error.
    assert!(matches!(
        token_service.revoke(&signed1).await,
        Err(ServiceError::TokenRevoked)
    ));

    // Verify accept token2, just-expired token.
    token_service.verify(&signed2).await?;
    token_service.revoke(&signed2).await?;

    // Try some invalid tokens
    for invalid_jwt in ["invalid", "", "invalid.invalid", "invalid.invalid.invalid"] {
        assert!(matches!(
            token_service.revoke(invalid_jwt).await,
            Err(ServiceError::InvalidJwt)
        ));

        assert!(matches!(
            token_service.verify(invalid_jwt).await,
            Err(ServiceError::InvalidJwt)
        ));
    }

    Ok(())
}
