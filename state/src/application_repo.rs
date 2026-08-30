use async_trait::async_trait;
use service::{
    dto, entity,
    id::{AccountId, ApplicationId},
    repo::{ApplicationRepo, ApplicationRepoTransaction, RepoResult, limits},
};
use sqlx::{PgPool, PgTransaction};

/// Application repository using PostgreSQL.
pub struct ApplicationRepoImpl {
    pool: PgPool,
}

impl ApplicationRepoImpl {
    /// Creates a new application repository.
    pub fn boxed_new(pool: PgPool) -> Box<Self> {
        Box::new(Self { pool })
    }
}

#[async_trait]
impl ApplicationRepo for ApplicationRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn ApplicationRepoTransaction>> {
        Ok(Box::new(
            ApplicationRepoTransactionImpl::begin(self.pool.clone()).await?,
        ))
    }

    async fn list_by_owner(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Vec<dto::repo::OwnedApplicationSummary>> {
        let applications = sqlx::query_as!(
            dto::repo::OwnedApplicationSummary,
            r#"SELECT name, application_id AS "application_id!: ApplicationId"
                FROM applications WHERE owner_account_id = $1"#,
            account_id as _
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(applications)
    }

    async fn list_redirect_urls(&self, application_id: ApplicationId) -> RepoResult<Vec<String>> {
        let redirect_urls = sqlx::query_scalar!(
            r"SELECT redirect_url
                FROM application_redirect_urls WHERE application_id = $1",
            application_id as _
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(redirect_urls)
    }

    async fn is_owned_by(
        &self,
        application_id: ApplicationId,
        account_id: AccountId,
    ) -> RepoResult<bool> {
        let is_owned_by = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                    SELECT 1 FROM applications
                        WHERE owner_account_id = $1 AND application_id = $2
                ) AS "exists!""#,
            account_id as _,
            application_id as _
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(is_owned_by)
    }

    async fn count_by_owner(&self, account_id: AccountId) -> RepoResult<usize> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM applications
                WHERE owner_account_id = $1"#,
            account_id as _
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count as usize)
    }

    async fn count_redirect_urls(&self, application_id: ApplicationId) -> RepoResult<usize> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM application_redirect_urls
                WHERE application_id = $1"#,
            application_id as _
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count as usize)
    }
}

struct ApplicationRepoTransactionImpl<'a> {
    tx: PgTransaction<'a>,
}

impl ApplicationRepoTransactionImpl<'_> {
    async fn begin(pool: PgPool) -> RepoResult<Self> {
        Ok(Self {
            tx: pool.begin().await?,
        })
    }
}

#[async_trait]
impl ApplicationRepoTransaction for ApplicationRepoTransactionImpl<'_> {
    async fn commit(self: Box<Self>) -> RepoResult<()> {
        self.tx.commit().await?;

        Ok(())
    }

    async fn insert(&mut self, application_entity: entity::Application) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO applications (
                    application_id, owner_account_id,
                    name, client_secret_hash
                ) SELECT $1, $2, $3, $4
                WHERE (
                    SELECT COUNT(*) FROM applications
                        WHERE owner_account_id = $2
                ) < $5",
            application_entity.application_id as _,
            application_entity.owner_account_id as _,
            application_entity.name,
            &application_entity.client_secret_hash,
            limits::application_repo::MAXIMUM_APPLICATION_COUNT as i64
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn set_client_secret_hash(
        &mut self,
        application_id: ApplicationId,
        client_secret_hash: &[u8; 32],
    ) -> RepoResult<()> {
        sqlx::query!(
            "UPDATE applications SET client_secret_hash = $2
                WHERE application_id = $1",
            application_id as _,
            client_secret_hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn set_name(&mut self, application_id: ApplicationId, name: &str) -> RepoResult<()> {
        sqlx::query!(
            "UPDATE applications SET name = $2 WHERE application_id = $1",
            application_id as _,
            name
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn delete(&mut self, application_id: ApplicationId) -> RepoResult<()> {
        sqlx::query!(
            "DELETE FROM applications WHERE application_id = $1",
            application_id as _
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn insert_redirect_url(
        &mut self,
        application_id: ApplicationId,
        redirect_url: &str,
    ) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO application_redirect_urls
                (application_id, redirect_url)
                SELECT $1, $2
                WHERE (
                    SELECT COUNT(*) FROM application_redirect_urls
                        WHERE application_id = $1
                ) < $3",
            application_id as _,
            redirect_url,
            limits::application_repo::MAXIMUM_REDIRECT_URL_COUNT as i64
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn delete_redirect_url(
        &mut self,
        application_id: ApplicationId,
        redirect_url: &str,
    ) -> RepoResult<()> {
        sqlx::query!(
            "DELETE FROM application_redirect_urls
                WHERE application_id = $1 AND redirect_url = $2",
            application_id as _,
            redirect_url
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
