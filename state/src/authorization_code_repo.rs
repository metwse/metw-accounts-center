use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use service::{
    id::{AccountId, ApplicationId},
    repo::{AuthorizationCodeRepo, RepoResult},
    util::random_authorization_code,
};

/// Authorization code repository using Redis.
pub struct AuthorizationCodeRepoImpl {
    con: MultiplexedConnection,
}

impl AuthorizationCodeRepoImpl {
    /// Creates a new token repository.
    pub async fn boxed_new(con_generator: &impl AsyncFn() -> MultiplexedConnection) -> Box<Self> {
        Box::new(Self {
            con: con_generator().await,
        })
    }
}

#[async_trait]
impl AuthorizationCodeRepo for AuthorizationCodeRepoImpl {
    async fn create(
        &self,
        account_id: AccountId,
        application_id: ApplicationId,
    ) -> RepoResult<String> {
        let authorization_code = random_authorization_code();
        let key = to_authorization_code_key(application_id, &authorization_code);

        let mut con = self.con.clone();
        con.set_ex::<'_, _, _, usize>(&key, account_id.to_string(), 60)
            .await?;

        Ok(authorization_code)
    }

    async fn consume(
        &self,
        application_id: ApplicationId,
        authorization_code: &str,
    ) -> RepoResult<Option<AccountId>> {
        let key = to_authorization_code_key(application_id, authorization_code);

        let mut con = self.con.clone();

        Ok(con
            .get_del::<'_, _, Option<String>>(&key)
            .await?
            .map(|id_str| id_str.parse().ok())
            .unwrap_or(None))
    }
}

/// One-time authorization code key.
pub fn to_authorization_code_key(
    application_id: ApplicationId,
    authorization_code: &str,
) -> String {
    format!(
        "authorization-code:{}:{}",
        application_id, authorization_code
    )
}
