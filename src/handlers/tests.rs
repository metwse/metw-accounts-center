use super::{AuthenticationHandler, HandlerError, HandlerResult, PersonalHandler};
use crate::{
    client::impls::MockMailClientImpl,
    dto, entity,
    handlers::AuthorizationHandler,
    repo::impls::{MockAccountRepoImpl, MockTokenRepoImpl},
    service::{AccountService, ServiceError, TokenService},
    token::TokenScope,
    util::templated_mails,
};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread")]
async fn account_creation() -> HandlerResult<()> {
    let account_repo = MockAccountRepoImpl::boxed_new();
    let token_repo = MockTokenRepoImpl::boxed_new();

    let account_service = Arc::new(AccountService::new(account_repo));
    let token_service = Arc::new(TokenService::new(token_repo, b"supersecret123"));
    let (emails, mail_client) = MockMailClientImpl::shared_new_with_emails();

    let authorization_handler =
        AuthorizationHandler::new(Arc::clone(&account_service), Arc::clone(&token_service));

    let authentication_handler = AuthenticationHandler::new(
        Arc::clone(&account_service),
        Arc::clone(&token_service),
        Arc::clone(&mail_client),
        "http://example.com",
    );

    let personal_handler = PersonalHandler::new(
        Arc::clone(&account_service),
        Arc::clone(&token_service),
        Arc::clone(&mail_client),
        "http://example.com",
    );

    let signup_dto = dto::request::Signup {
        username: "user".to_string(),
        email: "user@example.com".to_string(),
        password_hash: "passwd".to_string(),
        keys: dto::request::Keys {
            identity_key: vec![1],
            encrypted_private_key: vec![2],
            encrypted_master_key: vec![3],
        },
    };

    let mut signup_dto2 = signup_dto.clone();
    signup_dto2.username += "2";
    signup_dto2.email = "user2@example.com".to_string();

    let login_dto = dto::request::LoginWithUsername {
        username: "user".to_string(),
        password_hash: "passwd".to_string(),
    };

    let login_dto_email = dto::request::LoginWithEmail {
        email: "user@example.com".to_string(),
        password_hash: "passwd".to_string(),
    };

    // Sign up an account.
    let account_id = authentication_handler.signup(signup_dto).await?;
    let account2_id = authentication_handler.signup(signup_dto2).await?;

    personal_handler.me(account_id).await?;

    // Try to get non-existent user.
    assert!(personal_handler.me(entity::AccountId(0)).await.is_err());

    assert!(matches!(
        authentication_handler
            .login_by_username(login_dto.clone())
            .await,
        Err(HandlerError::Service(ServiceError::AccountNotVerified))
    ));

    // Check sent verification mail.
    {
        let emails = emails.lock().await;

        let templated_mails::Template::Signup {
            username,
            signup_jwt,
            ..
        } = &emails[&account_id][0]
        else {
            unreachable!()
        };

        assert!(username == "user");

        let signup_token1 = token_service.verify(signup_jwt).await?;

        // Now the account is verified and we can log into it.
        authorization_handler.auth(signup_jwt.clone()).await?;

        assert!(signup_token1.id == account_id);
        assert!(matches!(signup_token1.scope, TokenScope::Signup { .. }));

        let templated_mails::Template::Signup {
            signup_jwt: signup_jwt2,
            ..
        } = &emails[&account2_id][0]
        else {
            unreachable!()
        };

        authorization_handler.auth(signup_jwt2.clone()).await?;
    }

    // Try to log in with username & password.
    authentication_handler
        .auth(authentication_handler.login_by_username(login_dto).await?)
        .await?;

    token_service
        .verify(
            &authentication_handler
                .login_by_email(login_dto_email)
                .await?,
        )
        .await?;

    // Add another email to the account.
    personal_handler
        .add_email(account_id, "email2@example.com".to_string())
        .await?;

    // Cannot add already-taken emails
    assert!(matches!(
        personal_handler
            .add_email(account_id, "user2@example.com".to_string())
            .await,
        Err(HandlerError::Service(ServiceError::EmailTaken))
    ));

    // Try to add account2's email as primary mail
    assert!(matches!(
        personal_handler
            .set_primary_mail(account_id, "user2@example.com".to_string())
            .await,
        Err(HandlerError::Service(ServiceError::EmailNotFound))
    ));

    // Validate the new email.
    {
        let emails = emails.lock().await;

        let templated_mails::Template::AddEmail {
            email,
            add_email_jwt,
            ..
        } = &emails[&account_id][1]
        else {
            unreachable!()
        };

        let add_email_token = token_service.verify(add_email_jwt).await?;

        // Add the email.
        authorization_handler.auth(add_email_jwt.clone()).await?;

        assert!(add_email_token.id == account_id);
        assert!(matches!(add_email_token.scope, TokenScope::AddEmail { .. }));
        assert!(email == "email2@example.com");
    }

    // Change primary email.
    personal_handler
        .set_primary_mail(account_id, "email2@example.com".to_string())
        .await?;

    {
        let emails = emails.lock().await;

        let templated_mails::Template::SetPrimaryEmail {
            set_primary_mail_jwt,
            ..
        } = &emails[&account_id][2]
        else {
            unreachable!()
        };

        token_service.verify(set_primary_mail_jwt).await?;

        // Change the primary mail.
        authorization_handler
            .auth(set_primary_mail_jwt.clone())
            .await?;
    }

    // Delete the old email.
    personal_handler
        .delete_email(account_id, "user@example.com".to_string())
        .await?;

    // Cannot remove primary email.
    assert!(
        personal_handler
            .delete_email(account_id, "email2@example.com".to_string())
            .await
            .is_err()
    );

    Ok(())
}
