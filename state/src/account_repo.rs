use async_trait::async_trait;
use service::{
    dto, entity,
    id::AccountId,
    repo::{AccountRepo, AccountRepoTransaction, RepoResult},
};
use sqlx::{PgPool, PgTransaction, types::Json};

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

type ServerPasswordHashAlgorithmJson = sqlx::types::Json<entity::ServerPasswordHashAlgorithm>;
type ClientPasswordKdfJson = sqlx::types::Json<entity::ClientPasswordKdf>;

#[async_trait]
impl AccountRepo for AccountRepoImpl {
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AccountRepoTransaction>> {
        Ok(Box::new(
            AccountRepoTransactionImpl::begin(self.pool.clone()).await?,
        ))
    }

    async fn get_login_credentials_by_account_id(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        // TODO: Write this query using JOIN
        let Some(login) = sqlx::query!(
            r#"SELECT password_hash,
                    (SELECT is_email_verified FROM account_flags
                        WHERE account_flags.account_id = $1) AS "is_email_verified!",
                    server_password_hash_algorithm AS
                        "server_password_hash_algorithm: ServerPasswordHashAlgorithmJson"
                FROM accounts
                WHERE account_id = $1"#,
            account_id as _
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let login = dto::repo::OwnedLoginCredentials {
            account_id,
            is_email_verified: login.is_email_verified,
            password_hash: login.password_hash,
            server_password_hash_algorithm: login.server_password_hash_algorithm.0,
        };

        Ok(Some(login))
    }

    async fn get_login_credentials_by_email(
        &self,
        email: &str,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        let Some(login) = sqlx::query!(
            r#"SELECT account_id AS "account_id: AccountId", password_hash,
                    server_password_hash_algorithm AS
                        "server_password_hash_algorithm: ServerPasswordHashAlgorithmJson"
                FROM accounts
                WHERE account_id = (SELECT account_id FROM emails WHERE email = $1)"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let login = dto::repo::OwnedLoginCredentials {
            account_id: login.account_id,
            is_email_verified: true,
            password_hash: login.password_hash,
            server_password_hash_algorithm: login.server_password_hash_algorithm.0,
        };

        Ok(Some(login))
    }

    async fn get_login_credentials_by_username(
        &self,
        username: &str,
    ) -> RepoResult<Option<dto::repo::OwnedLoginCredentials>> {
        // TODO: Write this query using JOIN
        let Some(login) = sqlx::query!(
            r#"SELECT account_id AS "account_id: AccountId", password_hash,
                    (SELECT is_email_verified FROM account_flags
                        WHERE account_flags.account_id = accounts.account_id) AS "is_email_verified!",
                    server_password_hash_algorithm AS
                        "server_password_hash_algorithm: ServerPasswordHashAlgorithmJson"
                FROM accounts
                WHERE account_id = (SELECT account_id FROM usernames WHERE username = $1 AND expires_at IS NULL)"#,
            username
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let login = dto::repo::OwnedLoginCredentials {
            account_id: login.account_id,
            is_email_verified: login.is_email_verified,
            password_hash: login.password_hash,
            server_password_hash_algorithm: login.server_password_hash_algorithm.0,
        };

        Ok(Some(login))
    }

    async fn get_client_password_kdf_by_email(
        &self,
        email: &str,
    ) -> RepoResult<Option<entity::ClientPasswordKdf>> {
        let Some(client_password_kdf_json) = sqlx::query_scalar!(
            r#"SELECT client_password_kdf AS
                        "client_password_kdf: ClientPasswordKdfJson"
                FROM accounts
                WHERE account_id = (SELECT account_id FROM emails WHERE email = $1)"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        Ok(Some(client_password_kdf_json.0))
    }

    async fn get_client_password_kdf_by_username(
        &self,
        username: &str,
    ) -> RepoResult<Option<entity::ClientPasswordKdf>> {
        let Some(client_password_kdf_json) = sqlx::query_scalar!(
            r#"SELECT client_password_kdf AS
                        "client_password_kdf: ClientPasswordKdfJson"
                FROM accounts
                WHERE account_id = (SELECT account_id FROM usernames WHERE username = $1 AND expires_at IS NULL)"#,
            username
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        Ok(Some(client_password_kdf_json.0))
    }

    async fn get_client_password_kdf_by_account_id(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Option<entity::ClientPasswordKdf>> {
        let Some(client_password_kdf_json) = sqlx::query_scalar!(
            r#"SELECT client_password_kdf AS
                        "client_password_kdf: ClientPasswordKdfJson"
                FROM accounts
                WHERE account_id = $1"#,
            account_id as _
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        Ok(Some(client_password_kdf_json.0))
    }

    async fn get_primary_username(&self, account_id: AccountId) -> RepoResult<Option<String>> {
        let username = sqlx::query_scalar!(
            "SELECT username FROM usernames
                WHERE account_id = $1 AND is_primary = true",
            account_id as _
        )
        .fetch_optional(&self.pool)
        .await;

        Ok(username?)
    }

    async fn get_nonexpiring_username_aliases(
        &self,
        account_id: AccountId,
    ) -> RepoResult<Vec<String>> {
        let usernames = sqlx::query_scalar!(
            "SELECT username FROM usernames
                WHERE account_id = $1 AND is_primary = false AND expires_at IS NULL",
            account_id as _
        )
        .fetch_all(&self.pool)
        .await;

        Ok(usernames?)
    }

    async fn get_primary_email(&self, account_id: AccountId) -> RepoResult<Option<String>> {
        let email = sqlx::query_scalar!(
            "SELECT email FROM emails WHERE account_id = $1 AND is_primary = true",
            account_id as _
        )
        .fetch_optional(&self.pool)
        .await;

        Ok(email?)
    }

    async fn get_secondary_emails(&self, account_id: AccountId) -> RepoResult<Vec<String>> {
        let emails = sqlx::query_scalar!(
            "SELECT email FROM emails WHERE account_id = $1 AND is_primary = false",
            account_id as _
        )
        .fetch_all(&self.pool)
        .await;

        Ok(emails?)
    }

    async fn compare_and_set_primary_email(
        &self,
        account_id: AccountId,
        expected_primary_email: &str,
        new_primary_email: &str,
    ) -> RepoResult<bool> {
        let mut tx = self.pool.begin().await?;

        let result1 = sqlx::query!(
            "UPDATE emails SET is_primary = false
                WHERE account_id = $1 AND is_primary = true AND email = $2 AND
                      EXISTS(SELECT * FROM emails WHERE
                             account_id = $1 AND is_primary = false AND email = $3)",
            account_id as _,
            expected_primary_email,
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
            account_id as _,
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

    async fn delete_email_if_not_primary(
        &self,
        account_id: AccountId,
        email: &str,
    ) -> RepoResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM emails
                WHERE account_id = $1 AND is_primary = false AND email = $2",
            account_id as _,
            email
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    async fn is_username_taken(&self, username: &str) -> RepoResult<bool> {
        let is_username_taken = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                    SELECT * FROM usernames WHERE username = $1
                ) AS "exists!""#,
            username
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(is_username_taken)
    }

    async fn is_email_taken(&self, email: &str) -> RepoResult<bool> {
        let is_email_taken = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT * FROM emails WHERE email = $1) AS "exists!""#,
            email
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(is_email_taken)
    }

    async fn is_email_owned_by(&self, account_id: AccountId, email: &str) -> RepoResult<bool> {
        let is_email_owned_by = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                    SELECT * FROM emails WHERE account_id = $1 AND email = $2
                ) AS "exists!""#,
            account_id as _,
            email
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(is_email_owned_by)
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

    async fn lock(&mut self, account_id: AccountId) -> RepoResult<()> {
        sqlx::query!(
            "SELECT account_id FROM accounts WHERE account_id = $1 FOR UPDATE",
            account_id as _
        )
        .fetch_optional(&mut *self.tx)
        .await
        .ok();

        Ok(())
    }

    async fn insert(&mut self, account_entity: &entity::Account) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO accounts (
                    account_id,
                    client_password_kdf, server_password_hash_algorithm, password_hash,
                    master_key_kek_kdf, master_key_encryption_algorithm, encrypted_master_key,
                    master_key_id
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            account_entity.account_id as _,
            Json(&account_entity.client_password_kdf) as _,
            Json(&account_entity.server_password_hash_algorithm) as _,
            account_entity.password_hash,
            Json(&account_entity.master_key_kek_kdf) as _,
            Json(&account_entity.master_key_encryption_algorithm) as _,
            account_entity.encrypted_master_key,
            account_entity.master_key_id as _
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn insert_default_flags(&mut self, account_id: AccountId) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO account_flags (account_id, is_email_verified) VALUES ($1, false)",
            account_id as _
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn insert_email(
        &mut self,
        account_id: AccountId,
        email: &str,
        is_primary: bool,
    ) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO emails (account_id, email, is_primary)
                VALUES ($1, $2, $3)",
            account_id as _,
            email,
            is_primary,
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn insert_username(
        &mut self,
        account_id: AccountId,
        username: &str,
        is_primary: bool,
    ) -> RepoResult<()> {
        sqlx::query!(
            "INSERT INTO usernames (account_id, username, is_primary)
                VALUES ($1, $2, $3)",
            account_id as _,
            username,
            is_primary
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn set_email_verified_flag(
        &mut self,
        account_id: AccountId,
        is_email_verified: bool,
    ) -> RepoResult<()> {
        sqlx::query!(
            "UPDATE account_flags SET is_email_verified = $1 WHERE account_id = $2",
            is_email_verified,
            account_id as _
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn set_password_credentials(
        &mut self,
        account_id: AccountId,
        password_hash: &str,
        client_password_kdf: &entity::ClientPasswordKdf,
        server_password_hash_algorithm: &entity::ServerPasswordHashAlgorithm,
    ) -> RepoResult<()> {
        let client_password_kdf_json = Json(client_password_kdf);
        let server_password_hash_algorithm_json = Json(server_password_hash_algorithm);

        sqlx::query!(
            r#"UPDATE accounts SET
                    password_hash = $2,
                    client_password_kdf = $3,
                    server_password_hash_algorithm = $4
                WHERE account_id = $1"#,
            account_id as _,
            password_hash,
            client_password_kdf_json as _,
            server_password_hash_algorithm_json as _
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    async fn count_emails(&mut self, account_id: AccountId) -> RepoResult<usize> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM emails WHERE account_id = $1"#,
            account_id as _
        )
        .fetch_one(&mut *self.tx)
        .await?;

        Ok(count as usize)
    }
}
