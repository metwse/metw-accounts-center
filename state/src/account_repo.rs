use async_trait::async_trait;
use service::{
    dto,
    id::AccountId,
    repo::{AccountRepo, AccountRepoTransaction, RepoResult},
};
use sqlx::{PgPool, PgTransaction};

/// Account repository using PostgreSQL.
pub struct AccountRepoImpl {
    pool: PgPool,
}

impl AccountRepoImpl {
    /// Creates a new account repository.
    pub fn boxed_new(pool: PgPool) -> Box<Self> {
        Box::new(Self { pool })
    }
}

#[async_trait]
impl AccountRepo for AccountRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AccountRepoTransaction>> {
        Ok(Box::new(
            AccountRepoTransactionImpl::begin(self.pool.clone()).await?,
        ))
    }

    async fn get_login_credentials_by_email(
        &self,
        email: &str,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        let login = sqlx::query_as!(
            dto::repo::OwnedLoginCredentials,
            r#"SELECT id, password_hash, true AS "is_email_verified!"
                FROM accounts
                WHERE id = (SELECT account_id FROM emails WHERE email = $1)"#,
            email
        )
        .fetch_optional(&self.pool)
        .await;

        Ok(login?)
    }

    async fn get_login_credentials_by_username(
        &self,
        username: &str,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        let login = sqlx::query_as!(
            dto::repo::OwnedLoginCredentials,
            r#"SELECT id, password_hash,
                    (SELECT is_email_verified FROM account_flags
                        WHERE account_flags.id = accounts.id) AS "is_email_verified!"
                FROM accounts
                WHERE id = (SELECT account_id FROM usernames WHERE username = $1 AND expires_at IS NULL)"#,
            username
        )
        .fetch_optional(&self.pool)
        .await;

        Ok(login?)
    }

    async fn get_primary_username(&self, id: AccountId) -> RepoResult<Option<String>> {
        let username = sqlx::query_scalar!(
            "SELECT username FROM usernames
                WHERE account_id = $1 AND is_primary = true",
            id as _
        )
        .fetch_optional(&self.pool)
        .await;

        Ok(username?)
    }

    async fn get_nonexpiring_username_aliases(&self, id: AccountId) -> RepoResult<Vec<String>> {
        let usernames = sqlx::query_scalar!(
            "SELECT username FROM usernames
                WHERE account_id = $1 AND is_primary = false AND expires_at IS NULL",
            id as _
        )
        .fetch_all(&self.pool)
        .await;

        Ok(usernames?)
    }

    async fn get_primary_email(&self, id: AccountId) -> RepoResult<Option<String>> {
        let email = sqlx::query_scalar!(
            "SELECT email FROM emails WHERE account_id = $1 AND is_primary = true",
            id as _
        )
        .fetch_optional(&self.pool)
        .await;

        Ok(email?)
    }

    async fn get_secondary_emails(&self, id: AccountId) -> RepoResult<Vec<String>> {
        let emails = sqlx::query_scalar!(
            "SELECT email FROM emails WHERE account_id = $1 AND is_primary = false",
            id as _
        )
        .fetch_all(&self.pool)
        .await;

        Ok(emails?)
    }

    async fn get_keys(&self, id: AccountId) -> RepoResult<Option<dto::repo::OwnedKeys>> {
        let keys = sqlx::query_as!(
            dto::repo::OwnedKeys,
            "SELECT identity_key, encrypted_private_key, encrypted_master_key FROM accounts
                WHERE id = $1",
            id as _
        )
        .fetch_optional(&self.pool)
        .await;

        Ok(keys?)
    }

    async fn set_primary_email_if_current_is(
        &self,
        id: AccountId,
        current_primary_email: &str,
        new_primary_email: &str,
    ) -> RepoResult<bool> {
        let mut tx = self.pool.begin().await?;

        let result1 = sqlx::query!(
            "UPDATE emails SET is_primary = false
                WHERE account_id = $1 AND is_primary = true AND email = $2 AND
                      EXISTS(SELECT * FROM emails WHERE
                             account_id = $1 AND is_primary = false AND email = $3)",
            id as _,
            current_primary_email,
            new_primary_email
        )
        .execute(&mut *tx)
        .await?;

        if result1.rows_affected() == 0 {
            return Ok(false);
        }

        let result2 = sqlx::query!(
            "UPDATE emails SET is_primary = true
                WHERE account_id = $1 AND is_primary = false AND email = $2",
            id as _,
            new_primary_email
        )
        .execute(&mut *tx)
        .await?;

        if result2.rows_affected() == 0 {
            return Ok(false);
        }

        tx.commit().await?;

        Ok(true)
    }

    async fn remove_email_if_not_primary(&self, id: AccountId, email: &str) -> RepoResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM emails
                WHERE account_id = $1 AND is_primary = false AND email = $2",
            id as _,
            email
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    async fn is_username_taken(&self, username: &str) -> RepoResult<bool> {
        let is_username_taken = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT * FROM usernames WHERE username = $1)",
            username
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap();

        Ok(is_username_taken)
    }

    async fn is_email_taken(&self, email: &str) -> RepoResult<bool> {
        let is_email_taken = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT * FROM emails WHERE email = $1)",
            email
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap();

        Ok(is_email_taken)
    }

    async fn is_email_taken_by(&self, id: AccountId, email: &str) -> RepoResult<bool> {
        let is_email_taken = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT * FROM emails WHERE account_id = $1 AND email = $2)",
            id as _,
            email
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap();

        Ok(is_email_taken)
    }
}

struct AccountRepoTransactionImpl<'a> {
    tx: PgTransaction<'a>,
}

impl AccountRepoTransactionImpl<'_> {
    async fn begin(pool: PgPool) -> RepoResult<Self> {
        Ok(Self {
            tx: pool.begin().await?,
        })
    }
}

#[async_trait]
impl AccountRepoTransaction for AccountRepoTransactionImpl<'_> {
    async fn commit(self: Box<Self>) -> RepoResult<()> {
        self.tx.commit().await?;

        Ok(())
    }

    async fn upsert_account(
        &mut self,
        id: AccountId,
        password_hash: &str,
        keys: &dto::repo::Keys,
    ) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO accounts (id, password_hash, identity_key, encrypted_master_key, encrypted_private_key)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id)
                DO UPDATE SET password_hash = $2, identity_key = $3, encrypted_master_key = $4, encrypted_private_key = $5",
            id as _,
            password_hash,
            keys.identity_key,
            keys.encrypted_master_key,
            keys.encrypted_private_key
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn insert_default_flags(&mut self, id: AccountId) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO account_flags (id, is_email_verified) VALUES ($1, false)",
            id as _
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn add_email(&mut self, id: AccountId, email: &str, is_primary: bool) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO emails (account_id, email, is_primary)
                VALUES ($1, $2, $3)",
            id as _,
            email,
            is_primary
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn add_username(
        &mut self,
        id: AccountId,
        username: &str,
        is_primary: bool,
    ) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO usernames (account_id, username, is_primary)
                VALUES ($1, $2, $3)",
            id as _,
            username,
            is_primary
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn set_is_email_verified_flag(
        &mut self,
        id: AccountId,
        is_email_verified: bool,
    ) -> RepoResult<()> {
        sqlx::query!(
            "UPDATE account_flags SET is_email_verified = $1 WHERE id = $2",
            is_email_verified,
            id as _
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
