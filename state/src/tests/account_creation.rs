use service::{
    dto,
    repo::AccountRepo,
    service::{AccountService, ServiceError, ServiceResult},
    testutil::{random_email, random_username},
    util::password,
};
use std::assert_matches;

// This test uses *account service*, as covering each and every branch of
// account repository without a service would be extremely hard.
pub async fn account_creation(repo: Box<dyn AccountRepo>) -> ServiceResult<()> {
    let account_service = AccountService::new(repo);

    let acc1_username = random_username();
    let acc1_email = random_email();
    let acc2_username = random_username();
    let acc2_email = random_email();

    // This basic sign up request will be used for all accounts.
    let signup_dto = dto::request::Signup {
        username: acc1_username.to_string(),
        email: acc1_email.to_string(),
        client_password_hash: "paswd1".to_string(),
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
    signup_dto2.client_password_hash = "paswd2".to_string();

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
                client_password_hash: "paswd2".to_string(),
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
            client_password_hash: "paswd1".to_string(),
        })
        .await?;
    // Is id returned from login same with sign up?
    assert!(acc1_id_from_login == acc1_id);

    // Try logging with invalid credentials.
    assert_matches!(
        account_service
            .login_with_username(dto::request::LoginWithUsername {
                username: "invalid_username".to_string(),
                client_password_hash: "paswd2".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        account_service
            .login_with_username(dto::request::LoginWithUsername {
                username: acc1_username.to_string(),
                client_password_hash: "invalid_password".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );

    // Also try invalid emails.
    assert_matches!(
        account_service
            .login_with_email(dto::request::LoginWithEmail {
                email: "invalid_email".to_string(),
                client_password_hash: "paswd1".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        account_service
            .login_with_email(dto::request::LoginWithEmail {
                email: acc1_email.to_string(),
                client_password_hash: "invalid_password".to_string(),
            })
            .await,
        Err(ServiceError::InvalidCredentials)
    );

    Ok(())
}
