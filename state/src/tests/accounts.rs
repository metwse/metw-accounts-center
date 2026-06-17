use service::{
    dto,
    service::{AccountService, ServiceError, ServiceResult},
    testutil::{random_email, random_username},
};
use std::{assert_matches, sync::Arc};

// Those tests uses *account service*, as covering each and every branch of
// account repository without a service would be extremely hard.

// Returns username.
// Account credentials are:
//  username: <random>
//  client_password_hash: passwd
pub async fn account_creation(account_service: Arc<AccountService>) -> ServiceResult<&'static str> {
    let username = random_username();
    let username2 = random_username();
    let another_username = random_username();
    let email = random_email();
    let another_email = random_email();

    let mut signup_dto = dto::request::Signup {
        username: username.to_string(),
        email: email.to_string(),
        client_password_hash: "passwd".to_string(),
        keys: dto::request::Keys {
            identity_key: vec![1],
            encrypted_private_key: vec![2],
            encrypted_master_key: vec![3],
        },
    };

    let login_with_email_dto = dto::request::LoginWithEmail {
        email: email.to_string(),
        client_password_hash: "passwd".to_string(),
    };

    let login_with_username_dto = dto::request::LoginWithUsername {
        username: username.to_string(),
        client_password_hash: "passwd".to_string(),
    };

    let login_with_incorrect_password = dto::request::LoginWithUsername {
        username: username.to_string(),
        client_password_hash: "incorrect_passwd".to_string(),
    };

    // Sign up the account. User cannot log into the account as it is not
    // verified.
    assert!(!account_service.is_username_taken(username).await?);
    assert!(!account_service.is_email_taken(email).await?);
    let account_id = account_service.signup(&signup_dto).await?;
    assert!(account_service.is_username_taken(username).await?);
    assert!(!account_service.is_email_taken(email).await?);

    // We will not able to activate this account as the email will be taken
    // from the other account.
    signup_dto.username = username2.to_string();
    let taken_email_cannot_verify_account_id = account_service.signup(&signup_dto).await?;

    signup_dto.email = another_email.to_string();
    assert_matches!(
        account_service.signup(&signup_dto).await,
        Err(ServiceError::UsernameTaken)
    );

    assert_matches!(
        account_service
            .login_with_email(&login_with_email_dto)
            .await,
        Err(ServiceError::InvalidCredentials)
    );

    assert_matches!(
        account_service
            .login_with_username(&login_with_username_dto)
            .await,
        Err(ServiceError::AccountNotVerified)
    );

    // Complete sign up and enable the account. Now user can log into its
    // account.
    account_service
        .auth_complete_signup(account_id, email)
        .await?;
    assert!(account_service.is_email_taken(email).await?);

    assert!(
        account_service
            .login_with_email(&login_with_email_dto)
            .await?
            == account_id
    );
    assert!(
        account_service
            .login_with_username(&login_with_username_dto)
            .await?
            == account_id
    );
    assert_matches!(
        account_service
            .login_with_username(&login_with_incorrect_password)
            .await,
        Err(ServiceError::InvalidCredentials)
    );

    // Email taken!
    signup_dto.email = email.to_string();
    signup_dto.username = another_username.to_string();
    assert_matches!(
        account_service.signup(&signup_dto).await,
        Err(ServiceError::EmailTaken)
    );

    // The account we created, but we cannot verify. Hopefully, the account
    // will be garbage-collected.
    assert_matches!(
        account_service
            .auth_complete_signup(taken_email_cannot_verify_account_id, email)
            .await,
        Err(ServiceError::SignupCompleteFailed)
    );

    let me = account_service.me(account_id).await?;

    assert!(me.id == i64::from(account_id));
    assert!(me.username.unwrap() == username);
    assert!(me.email.unwrap() == email);
    assert!(me.secondary_emails.is_empty());
    assert!(me.username_aliases.is_empty());
    assert!(me.keys.identity_key == vec![1]);
    assert!(me.keys.encrypted_private_key == vec![2]);
    assert!(me.keys.encrypted_master_key == vec![3]);

    Ok(username)
}

// Only one account should be created.
pub async fn account_creation_data_race(
    account_service: Arc<AccountService>,
) -> ServiceResult<&'static str> {
    let username = random_username();
    let email = random_email();

    let signup_dto = dto::request::Signup {
        username: username.to_string(),
        email: email.to_string(),
        client_password_hash: "passwd".to_string(),
        keys: dto::request::Keys {
            identity_key: vec![1],
            encrypted_private_key: vec![2],
            encrypted_master_key: vec![3],
        },
    };

    let mut signup_futures = Vec::with_capacity(16);

    for _ in 0..16 {
        signup_futures.push(account_service.signup(&signup_dto));
    }

    let signup_results = futures_util::future::join_all(signup_futures).await;

    assert!(signup_results.iter().filter(|res| res.is_ok()).count() == 1);

    Ok(username)
}

pub async fn email_change(
    username: &'static str,
    account_service: Arc<AccountService>,
) -> ServiceResult<()> {
    let account_id = account_service
        .login_with_username(&dto::request::LoginWithUsername {
            username: username.to_string(),
            client_password_hash: "passwd".to_string(),
        })
        .await?;

    let current_primary_email = account_service
        .get_primary_email(account_id)
        .await?
        .unwrap();

    let email2 = random_email();
    let email3 = random_email();

    // Add the email.
    account_service.auth_add_email(account_id, email2).await?;

    // Adding the same email failed because we already added one.
    assert_matches!(
        account_service.auth_add_email(account_id, email2).await,
        Err(ServiceError::EmailAddFailed)
    );

    // email3 is not added yet.
    assert_matches!(
        account_service
            .auth_change_primary_email(account_id, &current_primary_email, email3)
            .await,
        Err(ServiceError::ChangePrimaryEmailFailed)
    );

    // email2 is not primary.
    assert_matches!(
        account_service
            .auth_change_primary_email(account_id, email2, &current_primary_email,)
            .await,
        Err(ServiceError::ChangePrimaryEmailFailed)
    );

    // Try to remove current primary email.
    assert_matches!(
        account_service
            .remove_email_if_not_primary(account_id, &current_primary_email)
            .await,
        Err(ServiceError::CannotDeletePrimaryEmail)
    );

    // Set the email2 primary.
    account_service
        .auth_change_primary_email(account_id, &current_primary_email, email2)
        .await?;

    account_service.auth_add_email(account_id, email3).await?;

    // Remove the old primary email.
    account_service
        .remove_email_if_not_primary(account_id, &current_primary_email)
        .await?;

    assert!(
        !account_service
            .is_email_taken_by(account_id, &current_primary_email)
            .await?
    );
    assert!(
        account_service
            .is_email_taken_by(account_id, email3)
            .await?
    );

    Ok(())
}
