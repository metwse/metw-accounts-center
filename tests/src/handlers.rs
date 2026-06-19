use crate::util::TestCtx;
use service::{
    dto,
    handlers::{
        AuthenticationHandler, AuthorizationHandler, HandlerError, HandlerResult,
        PendingActivationSessionHandler, SessionHandler,
    },
    service::ServiceError,
    testutil::{random_email, random_username},
    token::TokenScope,
    util::mails,
};
use std::assert_matches;

/// Completes sign up with pending activation session.
pub async fn retry_signup(ctx: &TestCtx) -> HandlerResult<()> {
    let (_, _, taken_email) = ctx.signup_and_verify_email("passwd1").await;

    // Create the account.
    let (account_id, username, email_unverified) = ctx.signup("passwd").await;

    // Log into the pending activation session.
    let pending_activation_session_jwt = ctx.login_with_username(username, "passwd").await?;
    assert!(
        ctx.login_with_email(email_unverified, "passwd")
            .await
            .is_err()
    );

    let login_account_id = AuthenticationHandler(ctx.state.clone())
        .auth_pending_activation_session(pending_activation_session_jwt)
        .await?;

    assert!(account_id == login_account_id);

    let me = SessionHandler(ctx.state.clone()).me(account_id).await?;
    assert!(me.username.is_some());
    // No primary email as the account is not verified yet.
    assert!(me.email.is_none());

    // Resend the signup email.
    assert_matches!(
        PendingActivationSessionHandler(ctx.state.clone())
            .retry_signup(
                account_id,
                dto::request::Email {
                    email: taken_email.to_string(),
                },
            )
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::EmailTaken)
    );

    let email = random_email();
    PendingActivationSessionHandler(ctx.state.clone())
        .retry_signup(
            account_id,
            dto::request::Email {
                email: email.to_string(),
            },
        )
        .await?;

    let mails::Template::ConfirmSignup {
        token: complete_signup_jwt,
        ..
    } = ctx.last_email(account_id).await
    else {
        unreachable!()
    };

    // Now the second email is added.
    AuthorizationHandler(ctx.state.clone())
        .auth(complete_signup_jwt)
        .await?;

    ctx.login_with_email(email, "passwd").await?;

    Ok(())
}

/// Sign up an account and log into it.
pub async fn signup_and_login(ctx: &TestCtx) -> HandlerResult<()> {
    let (account_id, username, email) = ctx.signup("passwd").await;

    let mails::Template::ConfirmSignup {
        token: complete_signup_jwt,
        ..
    } = ctx.last_email(account_id).await
    else {
        unreachable!()
    };

    let me = SessionHandler(ctx.state.clone()).me(account_id).await?;
    assert!(me.username.is_some());
    assert!(me.email.is_none());

    // Now the email is added.
    AuthorizationHandler(ctx.state.clone())
        .auth(complete_signup_jwt)
        .await?;

    let me = SessionHandler(ctx.state.clone()).me(account_id).await?;
    assert!(me.email.unwrap() == email);

    // Try logging in with username and password.
    let session_jwt_from_email = ctx.login_with_email(email, "passwd").await?;
    let session_jwt_from_username = ctx.login_with_username(username, "passwd").await?;

    assert!(
        AuthenticationHandler(ctx.state.clone())
            .auth_session(session_jwt_from_email.clone())
            .await?
            == account_id
    );
    assert!(
        AuthenticationHandler(ctx.state.clone())
            .auth_session(session_jwt_from_username.clone())
            .await?
            == account_id
    );

    // Check invalid credentials.
    assert_matches!(
        ctx.login_with_email(email, "invalid_passwd")
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        ctx.login_with_email("invalid@email.com", "passwd")
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        ctx.login_with_username(username, "invalid_passwd")
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        ctx.login_with_username("invalid_username", "passwd")
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::InvalidCredentials)
    );

    // Provide session tokens to authorization handler.
    assert_matches!(
        AuthorizationHandler(ctx.state.clone())
            .auth(session_jwt_from_email)
            .await
            .unwrap_err(),
        HandlerError::Unauthorized
    );
    // Previous AuthorizationHandler call revoked the token. If the JWTs from
    // username and email logins are the same, then this AuthorizationHandler
    // call will return Unauthorized.
    assert_matches!(
        AuthorizationHandler(ctx.state.clone())
            .auth(session_jwt_from_username)
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::TokenRevoked) | HandlerError::Unauthorized
    );

    Ok(())
}

/// Log out session.
pub async fn logout(ctx: &TestCtx) -> HandlerResult<()> {
    let (_, username, _) = ctx.signup_and_verify_email("passwd").await;

    let session_jwt = ctx.login_with_username(username, "passwd").await?;

    AuthenticationHandler(ctx.state.clone())
        .logout(session_jwt.clone())
        .await?;

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .logout(session_jwt.clone())
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::TokenRevoked)
    );

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .auth_session(session_jwt.clone())
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::TokenRevoked)
    );

    Ok(())
}

/// Try to sign up with already taken username or email.
pub async fn taken_username_or_email(ctx: &TestCtx) -> HandlerResult<()> {
    let (_, taken_username, taken_email) = ctx.signup_and_verify_email("passwd").await;
    let (_, another_taken_username, _) = ctx.signup("passwd").await;

    let mut signup_dto = dto::request::Signup {
        username: taken_username.to_string(),
        email: random_email().to_string(),
        client_password_hash: "passwd".to_string(),
        keys: dto::request::Keys {
            identity_key: vec![1],
            encrypted_master_key: vec![2],
            encrypted_private_key: vec![2],
        },
    };

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .signup(signup_dto.clone())
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::UsernameTaken),
    );

    signup_dto.username = another_taken_username.to_string();

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .signup(signup_dto.clone())
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::UsernameTaken),
    );

    signup_dto.username = random_username().to_string();
    signup_dto.email = taken_email.to_string();

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .signup(signup_dto.clone())
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::EmailTaken),
    );

    Ok(())
}

/// Change primary email and remove the old email.
pub async fn change_primary_email(ctx: &TestCtx) -> HandlerResult<()> {
    // Sign up an account.
    let (acccount_id, _, email) = ctx.signup_and_verify_email("passwd1").await;
    let (_, _, another_accounts_email) = ctx.signup_and_verify_email("passwd2").await;

    let new_email = random_email();
    SessionHandler(ctx.state.clone())
        .add_email(
            acccount_id,
            dto::request::Email {
                email: new_email.to_string(),
            },
        )
        .await?;

    // Cannot add already-taken emails
    assert_matches!(
        SessionHandler(ctx.state.clone())
            .add_email(
                acccount_id,
                dto::request::Email {
                    email: another_accounts_email.to_string()
                }
            )
            .await,
        Err(HandlerError::Service(ServiceError::EmailTaken))
    );

    // Try to add account2's email as primary mail
    assert_matches!(
        SessionHandler(ctx.state.clone())
            .set_primary_mail(
                acccount_id,
                dto::request::Email {
                    email: new_email.to_string()
                }
            )
            .await,
        Err(HandlerError::Service(ServiceError::EmailNotFound))
    );

    // Validate the new email.
    {
        let mails::Template::ConfirmNewEmail {
            email,
            token: add_email_jwt,
            ..
        } = ctx.last_email(acccount_id).await
        else {
            unreachable!()
        };

        let add_email_token = ctx.state.token_service.verify(&add_email_jwt).await?;

        // Add the email.
        AuthorizationHandler(ctx.state.clone())
            .auth(add_email_jwt.clone())
            .await?;

        assert!(add_email_token.id == acccount_id);
        assert_matches!(add_email_token.scope, TokenScope::AddEmail { .. });
        assert!(email == new_email);
    }

    // Change primary email.
    SessionHandler(ctx.state.clone())
        .set_primary_mail(
            acccount_id,
            dto::request::Email {
                email: new_email.to_string(),
            },
        )
        .await?;

    {
        let mails::Template::ConfirmPrimaryEmailChange {
            token: change_primary_mail_jwt,
            ..
        } = ctx.last_email(acccount_id).await
        else {
            unreachable!()
        };

        ctx.state
            .token_service
            .verify(&change_primary_mail_jwt)
            .await?;

        // Change the primary mail.
        AuthorizationHandler(ctx.state.clone())
            .auth(change_primary_mail_jwt.clone())
            .await?;
    }

    // Delete the old email.
    SessionHandler(ctx.state.clone())
        .delete_email(
            acccount_id,
            dto::request::Email {
                email: email.to_string(),
            },
        )
        .await?;

    // Cannot remove primary email.
    assert!(
        SessionHandler(ctx.state.clone())
            .delete_email(
                acccount_id,
                dto::request::Email {
                    email: new_email.to_string()
                }
            )
            .await
            .is_err()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        change_primary_email, logout, retry_signup, signup_and_login, taken_username_or_email,
    };
    use crate::util::{TestCtx, pg_pool_from_env, redis_client_from_env};
    use service::handlers::HandlerResult;
    use state::{AccountRepoImpl, TokenRepoImpl};

    async fn testsuite(ctx: &TestCtx) -> HandlerResult<()> {
        for _ in 0..4 {
            retry_signup(ctx).await?;
            signup_and_login(ctx).await?;
            logout(ctx).await?;
            taken_username_or_email(ctx).await?;
            change_primary_email(ctx).await?;
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    async fn mock_repo() -> HandlerResult<()> {
        testsuite(&TestCtx::new()).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    #[ignore]
    #[serial_test::serial]
    async fn repo() -> HandlerResult<()> {
        let pg_pool = pg_pool_from_env().await;
        let redis = redis_client_from_env().await;

        let account_repo = AccountRepoImpl::boxed_new(pg_pool);
        let token_repo = TokenRepoImpl::boxed_new(redis);

        let ctx = TestCtx::new()
            .with_account_repo(account_repo)
            .with_token_repo(token_repo);

        testsuite(&ctx).await
    }
}
