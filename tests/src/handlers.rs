use crate::util::TestState;
use service::{
    dto,
    handlers::{
        AuthenticationHandler, EmailVerificationSessionHandler, HandlerError, HandlerResult,
        SessionHandler, TokenActionHandler,
    },
    id::AccountId,
    repo::limits,
    service::ServiceError,
    testutil::{random_email, random_ipv6, random_username},
    token::TokenScope,
    util::emails,
};
use std::{assert_matches, iter, time::Duration};

/// Completes sign up with pending activation session.
pub async fn retry_signup(ctx: &TestState) -> HandlerResult<()> {
    let (_, _, taken_email) = ctx.signup_and_verify_email("passwd1").await;

    // Create the account.
    let (account_id, username, email_unverified) = ctx.signup("passwd").await;

    // Log into the pending activation session.
    let email_verification_session_jwt = ctx.login_with_username(username, "passwd").await?;
    assert!(
        ctx.login_with_email(email_unverified, "passwd")
            .await
            .is_err()
    );

    AuthenticationHandler(ctx.state.clone())
        .get_client_password_kdf(dto::request::AccountIdentifier::Email(
            dto::request::Email {
                email: email_unverified.into(),
            },
        ))
        .await
        .unwrap_err();

    AuthenticationHandler(ctx.state.clone())
        .get_client_password_kdf(dto::request::AccountIdentifier::Username(
            dto::request::Username {
                username: username.into(),
            },
        ))
        .await?;

    let login_account_id = AuthenticationHandler(ctx.state.clone())
        .auth_email_verification_session(email_verification_session_jwt)
        .await?;

    AuthenticationHandler(ctx.state.clone())
        .get_client_password_kdf(dto::request::AccountIdentifier::AccountId(login_account_id))
        .await?;

    assert!(account_id == login_account_id);

    let me = SessionHandler(ctx.state.clone())
        .get_current_account(account_id)
        .await?;
    assert!(me.username.is_some());
    // No primary email as the account is not verified yet.
    assert!(me.email.is_none());

    // Resend the signup email.
    assert_matches!(
        EmailVerificationSessionHandler(ctx.state.clone())
            .retry_signup(
                account_id,
                dto::request::RetrySignup {
                    email: taken_email.to_string(),
                    redirect_url: None
                },
                random_ipv6(),
                dto::request::Captcha {
                    captcha: "captcha".to_string()
                }
            )
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::EmailTaken)
    );

    let email = random_email();
    EmailVerificationSessionHandler(ctx.state.clone())
        .retry_signup(
            account_id,
            dto::request::RetrySignup {
                email: email.to_string(),
                redirect_url: None,
            },
            random_ipv6(),
            dto::request::Captcha {
                captcha: "captcha".to_string(),
            },
        )
        .await?;

    let emails::Template::ConfirmSignup {
        token: complete_signup_jwt,
        ..
    } = ctx.last_email(account_id).await
    else {
        unreachable!()
    };

    // Now the second email is added.
    TokenActionHandler(ctx.state.clone())
        .execute_token_action(
            dto::request::Token {
                token: complete_signup_jwt,
            },
            random_ipv6(),
        )
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    ctx.login_with_email(email, "passwd").await?;

    Ok(())
}

/// Sign up an account and log into it.
pub async fn signup_and_login(ctx: &TestState) -> HandlerResult<()> {
    let (account_id, username, email) = ctx.signup("passwd").await;

    let emails::Template::ConfirmSignup {
        token: complete_signup_jwt,
        ..
    } = ctx.last_email(account_id).await
    else {
        unreachable!()
    };

    let me = SessionHandler(ctx.state.clone())
        .get_current_account(account_id)
        .await?;
    assert!(me.username.is_some());
    assert!(me.email.is_none());

    // Now the email is added.
    TokenActionHandler(ctx.state.clone())
        .execute_token_action(
            dto::request::Token {
                token: complete_signup_jwt,
            },
            random_ipv6(),
        )
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let me = SessionHandler(ctx.state.clone())
        .get_current_account(account_id)
        .await?;
    assert!(me.email.unwrap() == email);

    // Try logging in with username and password.
    let (session_jwt_from_email, session_jwt_from_username, session_jwt_from_account_id) = tokio::try_join!(
        ctx.login_with_email(email, "passwd"),
        ctx.login_with_username(username, "passwd"),
        ctx.login_with_account_id(account_id, "passwd"),
    )?;

    for jwt in [
        session_jwt_from_email.clone(),
        session_jwt_from_username.clone(),
        session_jwt_from_account_id.clone(),
    ] {
        assert!(
            AuthenticationHandler(ctx.state.clone())
                .auth_session(jwt)
                .await?
                == account_id
        );
    }

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
        HandlerError::Service(ServiceError::AccountNotFound)
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
        HandlerError::Service(ServiceError::AccountNotFound)
    );
    assert_matches!(
        ctx.login_with_account_id(account_id, "invalid_passwd")
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::InvalidCredentials)
    );
    assert_matches!(
        ctx.login_with_account_id(AccountId::unique(), "passwd")
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::AccountNotFound)
    );

    // Provide session tokens to the token-action handler.
    assert_matches!(
        TokenActionHandler(ctx.state.clone())
            .execute_token_action(
                dto::request::Token {
                    token: session_jwt_from_email
                },
                random_ipv6()
            )
            .await
            .unwrap_err(),
        HandlerError::Unauthorized
    );
    // Previous TokenActionHandler call revoked the token. If the JWTs from
    // username and email logins are the same, then this TokenActionHandler
    // call will return Unauthorized.
    assert_matches!(
        TokenActionHandler(ctx.state.clone())
            .execute_token_action(
                dto::request::Token {
                    token: session_jwt_from_username
                },
                random_ipv6()
            )
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::TokenRevoked) | HandlerError::Unauthorized
    );

    Ok(())
}

/// Log out session.
pub async fn logout(ctx: &TestState) -> HandlerResult<()> {
    let (_, username, _) = ctx.signup_and_verify_email("passwd").await;

    let session_jwt = ctx.login_with_username(username, "passwd").await?;

    AuthenticationHandler(ctx.state.clone())
        .logout(dto::request::Token {
            token: session_jwt.clone(),
        })
        .await?;

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .logout(dto::request::Token {
                token: session_jwt.clone()
            })
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
pub async fn taken_username_or_email(ctx: &TestState) -> HandlerResult<()> {
    let (_, taken_username, taken_email) = ctx.signup_and_verify_email("passwd").await;
    let (_, another_taken_username, _) = ctx.signup("passwd").await;

    let mut signup_dto = dto::request::Signup {
        username: taken_username.to_string(),
        email: random_email().to_string(),
        password: dto::request::ClientDerivedPassword {
            base64_hash: "passwd".to_string(),
            pbkdf2_salt: "metw-accounts-center".into(),
            pbkdf2_iterations: 500_000,
            pbkdf2_length: 256,
        },
        redirect_url: None,
    };

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .signup(
                signup_dto.clone(),
                random_ipv6(),
                dto::request::Captcha {
                    captcha: "captcha".to_string()
                }
            )
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::UsernameTaken),
    );

    signup_dto.username = another_taken_username.to_string();
    signup_dto.email = random_email().to_string();

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .signup(
                signup_dto.clone(),
                random_ipv6(),
                dto::request::Captcha {
                    captcha: "captcha".to_string()
                }
            )
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::UsernameTaken),
    );

    signup_dto.username = random_username().to_string();
    signup_dto.email = taken_email.to_string();

    assert_matches!(
        AuthenticationHandler(ctx.state.clone())
            .signup(
                signup_dto.clone(),
                random_ipv6(),
                dto::request::Captcha {
                    captcha: "captcha".to_string()
                }
            )
            .await
            .unwrap_err(),
        HandlerError::Service(ServiceError::EmailTaken),
    );

    Ok(())
}

/// Change primary email and remove the old email.
pub async fn change_primary_email(ctx: &TestState) -> HandlerResult<()> {
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
            random_ipv6(),
            dto::request::Captcha {
                captcha: "captcha".to_string(),
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
                },
                random_ipv6(),
                dto::request::Captcha {
                    captcha: "captcha".to_string()
                }
            )
            .await,
        Err(HandlerError::Service(ServiceError::EmailTaken))
    );

    // Try to add account2's email as primary email
    assert_matches!(
        SessionHandler(ctx.state.clone())
            .change_primary_email(
                acccount_id,
                dto::request::Email {
                    email: new_email.to_string()
                },
                dto::request::Captcha {
                    captcha: "captcha".to_string()
                }
            )
            .await,
        Err(HandlerError::Service(ServiceError::EmailNotFound))
    );

    // Validate the new email.
    {
        let emails::Template::ConfirmNewEmail {
            email,
            token: add_email_jwt,
            ..
        } = ctx.last_email(acccount_id).await
        else {
            unreachable!()
        };

        let add_email_token = ctx.state.token_service.verify(&add_email_jwt).await?;

        // Add the email.
        TokenActionHandler(ctx.state.clone())
            .execute_token_action(
                dto::request::Token {
                    token: add_email_jwt.clone(),
                },
                random_ipv6(),
            )
            .await?;

        assert!(add_email_token.sub == acccount_id);
        assert_matches!(add_email_token.scope, TokenScope::AddEmail { .. });
        assert!(email == new_email);
    }

    // Change primary email.
    SessionHandler(ctx.state.clone())
        .change_primary_email(
            acccount_id,
            dto::request::Email {
                email: new_email.to_string(),
            },
            dto::request::Captcha {
                captcha: "captcha".to_string(),
            },
        )
        .await?;

    {
        let emails::Template::ConfirmPrimaryEmailChange {
            token: change_primary_email_jwt,
            ..
        } = ctx.last_email(acccount_id).await
        else {
            unreachable!()
        };

        ctx.state
            .token_service
            .verify(&change_primary_email_jwt)
            .await?;

        // Change the primary email.
        TokenActionHandler(ctx.state.clone())
            .execute_token_action(
                dto::request::Token {
                    token: change_primary_email_jwt.clone(),
                },
                random_ipv6(),
            )
            .await?;
    }

    // Delete the old email.
    SessionHandler(ctx.state.clone())
        .remove_email(
            acccount_id,
            dto::request::Email {
                email: email.to_string(),
            },
        )
        .await?;

    // Cannot remove primary email.
    assert!(
        SessionHandler(ctx.state.clone())
            .remove_email(
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

/// Try to excess limits on emails, applications etc.
pub async fn limits_data_race(test_state: &TestState) -> HandlerResult<()> {
    let (account_id, _, _) = test_state.signup_and_verify_email("123").await;

    macro_rules! repeat_closure {
        ($closure:expr, $n:expr, $expect_ok_n:expr) => {
            assert_eq!(
                futures_util::future::join_all(iter::repeat_with($closure).take($n))
                    .await
                    .iter()
                    .filter(|res| res.is_ok())
                    .count(),
                $expect_ok_n
            );
        };
    }

    repeat_closure!(
        || {
            test_state
                .state
                .account_service
                .confirm_email_addition(account_id, random_email())
        },
        limits::account_repo::MAXIMUM_EMAIL_COUNT.pow(2),
        limits::account_repo::MAXIMUM_EMAIL_COUNT - 1
    );

    repeat_closure!(
        || {
            test_state
                .state
                .application_service
                .create(account_id, random_username())
        },
        limits::application_repo::MAXIMUM_APPLICATION_COUNT.pow(2),
        limits::application_repo::MAXIMUM_APPLICATION_COUNT
    );

    let (account2_id, _, _) = test_state.signup_and_verify_email("123").await;
    let dto::service::CreatedApplication { application_id, .. } = test_state
        .state
        .application_service
        .create(account2_id, random_username())
        .await?;

    repeat_closure!(
        || {
            test_state
                .state
                .application_service
                .add_redirect_url(application_id, random_username())
        },
        limits::application_repo::MAXIMUM_REDIRECT_URL_COUNT.pow(2),
        limits::application_repo::MAXIMUM_REDIRECT_URL_COUNT
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        change_primary_email, limits_data_race, logout, retry_signup, signup_and_login,
        taken_username_or_email,
    };
    use crate::util::{TestState, pg_pool_from_env, redis_con_generator_from_env};
    use service::handlers::HandlerResult;
    use state::{AccountRepoImpl, ApplicationRepoImpl, TokenRepoImpl};

    async fn testsuite(ctx: &TestState) -> HandlerResult<()> {
        let run_tests = async || {
            tokio::try_join!(
                retry_signup(ctx),
                signup_and_login(ctx),
                logout(ctx),
                taken_username_or_email(ctx),
                change_primary_email(ctx),
                limits_data_race(ctx),
            )
            .unwrap();
        };

        tokio::join!(
            run_tests(),
            run_tests(),
            run_tests(),
            run_tests(),
            run_tests()
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    async fn mock_repo() -> HandlerResult<()> {
        testsuite(&TestState::new()).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[test_log::test]
    #[ignore]
    async fn repo() -> HandlerResult<()> {
        let pg_pool = pg_pool_from_env().await;
        let con_generator = redis_con_generator_from_env().await;

        let account_repo = AccountRepoImpl::boxed_new(pg_pool.clone());
        let application_repo = ApplicationRepoImpl::boxed_new(pg_pool);
        let token_repo = TokenRepoImpl::boxed_new(&con_generator).await;

        let ctx = TestState::new()
            .with_account_repo(account_repo)
            .with_application_repo(application_repo)
            .with_token_repo(token_repo);

        testsuite(&ctx).await
    }
}
