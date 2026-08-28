use async_trait::async_trait;
use service::{
    dto, entity,
    id::{AccountId, AppId},
    repo::{AppRepo, AppRepoTransaction, RepoResult},
};
use sqlx::{PgPool, PgTransaction};

/// Application repository using PostgreSQL.
pub struct AppRepoImpl {
    pool: PgPool,
}

impl AppRepoImpl {
    /// Creates a new application repository.
    pub fn boxed_new(pool: PgPool) -> Box<Self> {
        Box::new(Self { pool })
    }
}

#[async_trait]
impl AppRepo for AppRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AppRepoTransaction>> {
        Ok(Box::new(
            AppRepoTransactionImpl::begin(self.pool.clone()).await?,
        ))
    }

    async fn get_apps(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Vec<dto::repo::OwnedBasicAppInfo>> {
        let apps = sqlx::query_as!(
            dto::repo::OwnedBasicAppInfo,
            r#"SELECT name, app_id AS "app_id!: AppId" FROM apps WHERE owner_account_id = $1"#,
            account_id as _
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(apps)
    }

    async fn get_redirect_urls(&self, app_id: AppId) -> RepoResult<Vec<String>> {
        let redirect_urls = sqlx::query_scalar!(
            r"SELECT redirect_url FROM app_redirect_urls WHERE app_id = $1",
            app_id as _
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(redirect_urls)
    }

    async fn is_app_owned_by(&self, account_id: AccountId, app_id: AppId) -> RepoResult<bool> {
        let is_app_owned_by = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                    SELECT 1 FROM apps WHERE owner_account_id = $1 AND app_id = $2
                ) AS "exists!""#,
            account_id as _,
            app_id as _
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(is_app_owned_by)
    }
}

struct AppRepoTransactionImpl<'a> {
    tx: PgTransaction<'a>,
}

impl AppRepoTransactionImpl<'_> {
    async fn begin(pool: PgPool) -> RepoResult<Self> {
        Ok(Self {
            tx: pool.begin().await?,
        })
    }
}

#[async_trait]
impl AppRepoTransaction for AppRepoTransactionImpl<'_> {
    async fn commit(self: Box<Self>) -> RepoResult<()> {
        self.tx.commit().await?;

        Ok(())
    }

    async fn insert_app(&mut self, app: entity::App) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO apps (
                    app_id, owner_account_id,
                    name, client_secret_hash
                ) VALUES ($1, $2, $3, $4)",
            app.app_id as _,
            app.owner_account_id as _,
            app.name,
            &app.client_secret_hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn update_client_secret_hash(
        &mut self,
        app_id: AppId,
        client_secret_hash: &[u8; 32],
    ) -> RepoResult<()> {
        sqlx::query!(
            "UPDATE apps SET client_secret_hash = $2 WHERE app_id = $1",
            app_id as _,
            client_secret_hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn delete_app(&mut self, app_id: AppId) -> RepoResult<()> {
        sqlx::query!("DELETE FROM apps WHERE app_id = $1", app_id as _)
            .execute(&mut *self.tx)
            .await?;

        Ok(())
    }

    async fn add_redirect_url(&mut self, app_id: AppId, redirect_url: &str) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO app_redirect_urls (app_id, redirect_url) VALUES ($1, $2)",
            app_id as _,
            redirect_url
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn remove_redirect_url(&mut self, app_id: AppId, redirect_url: &str) -> RepoResult<()> {
        sqlx::query!(
            "DELETE FROM app_redirect_urls WHERE app_id = $1 AND redirect_url = $2",
            app_id as _,
            redirect_url
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
