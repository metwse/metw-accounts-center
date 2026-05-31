use super::{AccountService, ServiceResult};
use crate::{
    dto,
    repo::impls::{MockAccountRepoImpl, MockTokenRepoImpl},
    service::{ServiceError, TokenService},
    testutil::{random_email, random_username},
    token::{Token, TokenScope},
    util::{JsonWebSignature, password},
};
use chrono::Utc;
use futures_util::future::join_all;
use std::{assert_matches, sync::Arc, time::Duration};

#[tokio::test(flavor = "multi_thread")] // multi_thread used to test Send+Sync
#[test_log::test]
async fn account_creation_mock_mt() -> ServiceResult<()> {
    let repo = MockAccountRepoImpl::boxed_new();

    let account_service = AccountService::new(repo);

    let acc1_username = random_username();
    let acc1_email = random_email();
    let acc2_username = random_username();
    let acc2_email = random_email();

    // This basic sign up request will be used for all accounts.
    let signup_dto = dto::request::Signup {
        username: acc1_username.to_string(),
        email: acc1_email.to_string(),
        password_hash: "paswd1".to_string(),
        keys: dto::request::Keys {
            identity_key: vec![1],
            encrypted_private_key: vec![2],
            encrypted_master_key: vec![3],
        },
    };

    let acc1_id = account_service.signup(signup_dto.clone()).await?;

    let mut signup_dto2 = signup_dto.clone();
    signup_dto2.username = acc2_username.to_string();
    signup_dto2.email = acc2_email.to_string();
    signup_dto2.password_hash = "paswd2".to_string();

    let acc2_id = account_service.signup(signup_dto2.clone()).await?;

    // Try to register an account with already-taken username.
    let mut already_taken_username = signup_dto.clone();
    already_taken_username.email = "...".to_string();
    assert_matches!(
        account_service.signup(already_taken_username).await,
        Err(ServiceError::UsernameTaken)
    );

    // Try to log into user2 account, but it is not verified.
    assert_matches!(
        account_service
            .login_with_username(dto::request::LoginWithUsername {
                username: acc2_username.to_string(),
                password_hash: "paswd2".to_string(),
            })
            .await,
        Err(ServiceError::AccountNotVerified)
    );

    // Get /me.
    account_service.me(acc2_id).await?;

    // Get /me from non-existent account.
    assert_matches!(
        account_service.me(0.into()).await,
        Err(ServiceError::AccountNotFound)
    );

    {
        // Dive into `repo` layer.
        let repo = account_service.repo();

        // Validate account creation.
        assert!(repo.get_primary_username(acc1_id).await?.unwrap() == acc1_username);
        assert!(
            password::check(
                "paswd1".to_string(),
                repo.get_login_by_username(acc1_username)
                    .await?
                    .unwrap()
                    .password_hash
            )
            .await
        );
        // Email should not be added as we did not verified it.
        assert!(repo.get_login_by_email(acc1_email).await?.is_none());
        assert!(repo.get_primary_username(acc2_id).await?.unwrap() == acc2_username);
        assert!(
            repo.get_keys(acc2_id).await?.unwrap()
                == dto::repo::Keys {
                    identity_key: vec![1],
                    encrypted_private_key: vec![2],
                    encrypted_master_key: vec![3],
                }
        );

        // Add email to the account, and elaborate abstraction to account_service
        // again.
        let mut transaction = repo.begin_transaction().await?;
        transaction.add_email(acc1_id, acc1_email, true).await?;
        transaction.set_verified_flag(acc1_id, true).await?;
        transaction.commit().await?;
    }

    // Try to register an account with already-taken email.
    let mut already_taken_email = signup_dto.clone();
    already_taken_email.username = random_username().to_string();
    assert_matches!(
        account_service.signup(already_taken_email).await,
        Err(ServiceError::EmailTaken)
    );

    // Log into user1 account.
    let acc1_id_from_login = account_service
        .login_with_email(dto::request::LoginWithEmail {
            email: acc1_email.to_string(),
            password_hash: "paswd1".to_string(),
        })
        .await?;
    // Is id returned from login same with sign up?
    assert!(acc1_id_from_login == acc1_id);

    // Try logging with invalid credentials.
    assert_matches!(
        account_service
            .login_with_username(dto::request::LoginWithUsername {
                username: "invalid_username".to_string(),
                password_hash: "paswd2".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        account_service
            .login_with_username(dto::request::LoginWithUsername {
                username: acc1_username.to_string(),
                password_hash: "invalid_password".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );

    // Also try invalid emails.
    assert_matches!(
        account_service
            .login_with_email(dto::request::LoginWithEmail {
                email: "invalid_email".to_string(),
                password_hash: "paswd1".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        account_service
            .login_with_email(dto::request::LoginWithEmail {
                email: acc1_email.to_string(),
                password_hash: "invalid_password".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
#[serial_test::serial]
async fn token_service() -> ServiceResult<()> {
    let repo = MockTokenRepoImpl::boxed_new();

    let token_service = TokenService::new(repo, "supersecret1234".into());

    let token1 = Token::new(
        0.into(),
        TokenScope::Authenticate,
        Duration::from_secs(1000),
    );

    let token2 = Token::new(0.into(), TokenScope::Authenticate, Duration::from_secs(0));

    let token3 = Token::new(2.into(), TokenScope::Authenticate, Duration::from_secs(40));

    let signed1 = token_service.sign(&token1);
    let signed2 = token_service.sign(&token2);
    let signed3 = token_service.sign(&token3);
    let signed3_invalid = signed3.clone() + "a";

    // token1 should be valid.
    token_service.verify(&signed1).await?;

    // Revoke token1 and check revocation status.
    token_service.revoke(&signed1).await?;
    assert_matches!(
        token_service.verify(&signed1).await,
        Err(ServiceError::TokenRevoked)
    );
    // Revocation of already-revoked token returns error.
    assert_matches!(
        token_service.revoke(&signed1).await,
        Err(ServiceError::TokenRevoked)
    );

    // Do not allow token2, just-expired.
    assert_matches!(
        token_service.verify(&signed2).await,
        Err(ServiceError::InvalidJwt)
    );

    token_service.verify(&signed3).await?;
    // Do not allow token3, invalid signature.
    assert_matches!(
        token_service.verify(&signed3_invalid).await,
        Err(ServiceError::InvalidJwt)
    );

    // Try some invalid tokens
    for invalid_jwt in ["invalid", "", "invalid.invalid", "invalid.invalid.invalid"] {
        assert_matches!(
            token_service.revoke(invalid_jwt).await,
            Err(ServiceError::InvalidJwt)
        );

        assert_matches!(
            token_service.verify(invalid_jwt).await,
            Err(ServiceError::InvalidJwt)
        );
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
#[serial_test::serial]
async fn token_service_expired() -> ServiceResult<()> {
    let repo = MockTokenRepoImpl::boxed_new();

    let token_service = TokenService::new(repo, "supersecret1234".into());

    let token = Token::new(3.into(), TokenScope::Authenticate, Duration::from_secs(40));

    let signed = token_service.sign(&token);

    // Expire token4
    JsonWebSignature::inject_now(Some(Utc::now() - Duration::from_secs(40)));
    assert_matches!(
        token_service.verify(&signed).await,
        Err(ServiceError::InvalidJwt)
    );

    JsonWebSignature::inject_now(None);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
#[serial_test::serial]
async fn token_service_toctou() -> ServiceResult<()> {
    let repo = MockTokenRepoImpl::boxed_new();

    let token_service = Arc::new(TokenService::new(repo, "supersecret1234".into()));

    let token = Token::new(3.into(), TokenScope::Authenticate, Duration::from_secs(40));

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
