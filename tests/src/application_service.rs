use crate::util::TestState;
use service::{handlers::HandlerResult, testutil::random_username};

/// Create an account and register an application for it.
pub async fn create_service_and_check_ownership(ctx: &TestState) -> HandlerResult<()> {
    let (account1_id, _, _) = ctx.signup_and_verify_email("123").await;
    let (account2_id, _, _) = ctx.signup_and_verify_email("123").await;

    let app1_name = random_username();
    let app1 = ctx
        .state
        .application_service
        .create(account1_id, app1_name)
        .await?;

    let app2_name = random_username();
    let app2 = ctx
        .state
        .application_service
        .create(account2_id, app2_name)
        .await?;

    let account1_apps = ctx
        .state
        .application_service
        .list_owned_by(account1_id)
        .await?;
    assert_eq!(account1_apps.len(), 1);
    assert_eq!(account1_apps[0].name, app1_name);

    assert!(
        ctx.state
            .application_service
            .is_owned_by(app1.application_id, account1_id)
            .await?
    );
    assert!(
        !ctx.state
            .application_service
            .is_owned_by(app2.application_id, account1_id)
            .await?
    );
    assert!(
        !ctx.state
            .application_service
            .is_owned_by(app1.application_id, account2_id)
            .await?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        application_service::create_service_and_check_ownership,
        util::{TestState, pg_pool_from_env, redis_con_generator_from_env},
    };
    use service::handlers::HandlerResult;
    use state::{AccountRepoImpl, ApplicationRepoImpl, TokenRepoImpl};

    async fn testsuite(ctx: &TestState) -> HandlerResult<()> {
        create_service_and_check_ownership(ctx).await?;

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
