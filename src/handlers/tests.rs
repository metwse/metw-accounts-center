use super::{HandlerError, HandlerResult};
use crate::{
    service::ServiceError,
    testutil::{TestCtx, random_email},
    token::TokenScope,
    util::templated_mails,
};

#[tokio::test(flavor = "multi_thread")]
#[test_log::test]
async fn account_creation() -> HandlerResult<()> {
    let ctx = TestCtx::new();

    // Sign up an account.
    let (acc1_id, acc1_username, acc1_email) = ctx.signup("passwd1").await;
    let (acc2_id, acc2_username, acc2_email) = ctx.signup("passwd2").await;

    ctx.personal_handler.me(acc1_id).await?;

    // Try to get non-existent user.
    assert!(matches!(
        ctx.login_with_username(acc1_username, "passwd1").await,
        Err(HandlerError::Service(ServiceError::AccountNotVerified))
    ));

    // Check sent verification mail.
    {
        let templated_mails::Template::Signup {
            username,
            signup_jwt,
            ..
        } = ctx.last_email(acc1_id).await
        else {
            unreachable!()
        };

        assert!(username == acc1_username);

        let signup_token = ctx.token_service.verify(&signup_jwt).await?;

        // Now the account is verified and we can log into it.
        ctx.authorization_handler.auth(signup_jwt.clone()).await?;

        assert!(signup_token.id == acc1_id);
        assert!(matches!(signup_token.scope, TokenScope::Signup { .. }));

        let templated_mails::Template::Signup {
            username: username2,
            signup_jwt: signup_jwt2,
            ..
        } = ctx.last_email(acc2_id).await
        else {
            unreachable!()
        };

        assert!(username2 == acc2_username);

        // Provide authorization token to authentication handler
        assert!(matches!(
            ctx.authentication_handler.auth(signup_jwt2.clone()).await,
            Err(HandlerError::Unauthorized)
        ));

        ctx.authorization_handler.auth(signup_jwt2.clone()).await?;
    }

    // Try to log in with username & password.
    ctx.authentication_handler
        .auth(ctx.login_with_email(acc1_email, "passwd1").await?)
        .await?;

    ctx.authentication_handler
        .auth(ctx.login_with_username(acc1_username, "passwd1").await?)
        .await?;

    // Provide authorization token to authentication handler
    assert!(matches!(
        ctx.authorization_handler
            .auth(ctx.login_with_username(acc1_username, "passwd1").await?)
            .await,
        Err(HandlerError::Unauthorized)
    ));

    // Add another email to the account.
    let new_email = random_email();
    ctx.personal_handler
        .add_email(acc1_id, new_email.to_string())
        .await?;

    // Cannot add already-taken emails
    assert!(matches!(
        ctx.personal_handler
            .add_email(acc1_id, acc2_email.to_string())
            .await,
        Err(HandlerError::Service(ServiceError::EmailTaken))
    ));

    // Try to add account2's email as primary mail
    assert!(matches!(
        ctx.personal_handler
            .set_primary_mail(acc1_id, acc2_email.to_string())
            .await,
        Err(HandlerError::Service(ServiceError::EmailNotFound))
    ));

    // Validate the new email.
    {
        let templated_mails::Template::AddEmail {
            email,
            add_email_jwt,
            ..
        } = ctx.last_email(acc1_id).await
        else {
            unreachable!()
        };

        let add_email_token = ctx.token_service.verify(&add_email_jwt).await?;

        // Add the email.
        ctx.authorization_handler
            .auth(add_email_jwt.clone())
            .await?;

        assert!(add_email_token.id == acc1_id);
        assert!(matches!(add_email_token.scope, TokenScope::AddEmail { .. }));
        assert!(email == new_email);
    }

    // Change primary email.
    ctx.personal_handler
        .set_primary_mail(acc1_id, new_email.to_string())
        .await?;

    {
        let templated_mails::Template::SetPrimaryEmail {
            set_primary_mail_jwt,
            ..
        } = ctx.last_email(acc1_id).await
        else {
            unreachable!()
        };

        ctx.token_service.verify(&set_primary_mail_jwt).await?;

        // Change the primary mail.
        ctx.authorization_handler
            .auth(set_primary_mail_jwt.clone())
            .await?;
    }

    // Delete the old email.
    ctx.personal_handler
        .delete_email(acc1_id, acc1_email.to_string())
        .await?;

    // Cannot remove primary email.
    assert!(
        ctx.personal_handler
            .delete_email(acc1_id, new_email.to_string())
            .await
            .is_err()
    );

    Ok(())
}
