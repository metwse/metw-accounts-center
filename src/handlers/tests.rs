use super::HandlerResult;
use crate::{
    client::impls::MockMailClientImpl,
    dto, entity,
    handlers::{AccountHandler, AuthenticationHandler, HandlerError},
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

    let authentication_handler = AuthenticationHandler::new(
        Arc::clone(&account_service),
        Arc::clone(&token_service),
        Arc::clone(&mail_client),
        "http://example.com",
    );

    let account_handler = AccountHandler::new(
        Arc::clone(&account_service),
        Arc::clone(&token_service),
        Arc::clone(&mail_client),
    );

    // Sign up an account.
    let account_id = authentication_handler
        .signup(dto::request::Signup {
            username: "user".to_string(),
            email: "user@example.com".to_string(),
            password_hash: "passwd".to_string(),
            keys: dto::request::Keys {
                identity_key: vec![1],
                encrypted_private_key: vec![2],
                encrypted_master_key: vec![3],
            },
        })
        .await?;

    account_handler.me(account_id).await?;

    // Try to get non-existent user.
    assert!(matches!(
        account_handler.me(entity::AccountId(0)).await,
        Err(..)
    ));

    // Check sent verification mail.
    {
        let emails = emails.lock().await;

        let templated_mails::Template::Signup {
            username,
            signup_jwt,
            ..
        } = &emails[&account_id][0];

        assert!(username == "user");

        let signup_token = token_service.verify(signup_jwt).await?;
        assert!(signup_token.id == account_id);
        assert!(matches!(signup_token.scopes[0], TokenScope::Signup { .. }));
    }

    assert!(matches!(
        authentication_handler
            .login_by_username(dto::request::LoginWithUsername {
                username: "user".to_string(),
                password_hash: "passwd".to_string(),
            })
            .await,
        Err(HandlerError::Service(ServiceError::AccountNotVerified))
    ));

    /* TODO: add account verification
    // Try to log in with username & password.
    token_service
        .verify(
            &authentication_handler
                .login_by_username(dto::request::LoginWithUsername {
                    username: "user".to_string(),
                    password_hash: "passwd".to_string(),
                })
                .await?,
        )
        .await?;
    */

    Ok(())
}
