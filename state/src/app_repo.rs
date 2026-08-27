use async_trait::async_trait;
use service::{
    dto,
    id::{AccountId, AppId},
    repo::{AppRepo, AppRepoTransaction, RepoResult},
};
use sqlx::PgPool;

/// Application repository using PostgreSQL.
pub struct AppRepoImpl {
    _pool: PgPool,
}

impl AppRepoImpl {
    /// Creates a new application repository.
    pub fn boxed_new(pool: PgPool) -> Box<Self> {
        Box::new(Self { _pool: pool })
    }
}

#[async_trait]
impl AppRepo for AppRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AppRepoTransaction>> {
        todo!()
    }

    async fn get_apps(
        &self,
        _account_id: AccountId,
    ) -> RepoResult<Vec<dto::repo::OwnedBasicAppInfo>> {
        todo!()
    }

    async fn get_redirect_urls(&self, _app_id: AppId) -> RepoResult<Vec<String>> {
        todo!()
    }

    async fn is_app_owned_by(&self, _account_id: AccountId, _app_id: AppId) -> RepoResult<bool> {
        todo!()
    }
}
