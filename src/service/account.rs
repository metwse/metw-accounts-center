use super::{ServiceError, ServiceResult};
use crate::{
    dto, entity,
    repo::AccountRepo,
    snowflake,
    util::{check_password, hash_password},
};

/// Account state.
pub struct AccountService {
    pub(super) repo: Box<dyn AccountRepo>,
}

impl AccountService {
    /// Creates a new account service.
    pub fn new(repo: Box<dyn AccountRepo>) -> Self {
        Self { repo }
    }

    /// Signup a new account
    pub async fn signup(
        &self,
        signup_dto: dto::request::Signup,
    ) -> ServiceResult<entity::AccountId> {
        let password_hash = hash_password(signup_dto.password_hash).await;

        let mut transaction = self.repo.begin_transaction().await?;

        let keys = dto::repo::Keys {
            identity_key: signup_dto.keys.identity_key,
            encrypted_private_key: signup_dto.keys.encrypted_private_key,
            encrypted_master_key: signup_dto.keys.encrypted_master_key,
        };

        let id = entity::AccountId(snowflake());

        transaction
            .upsert_account(id, &password_hash, &keys)
            .await?;

        if !transaction.add_username(id, &signup_dto.username).await? {
            return Err(ServiceError::UsernameTaken);
        }

        if !transaction
            .set_primary_username(id, &signup_dto.username, true)
            .await?
        {
            // I cannot imagine in which conditions this branch is executed.
            // Most probably this is unreachable.
            return Err(ServiceError::UnexceptedError("could not take username"));
        }

        transaction.commit().await?;

        Ok(id)
    }

    /// Log into the account
    pub async fn login_with_email(
        &self,
        credentials: dto::request::LoginWithEmail,
    ) -> ServiceResult<entity::AccountId> {
        let login = if let Some(login) = self.repo.get_login_by_email(&credentials.email).await? {
            login
        } else {
            return Err(ServiceError::InvalidCredentials);
        };

        if check_password(credentials.password_hash, login.password_hash).await {
            Ok(login.id)
        } else {
            Err(ServiceError::InvalidCredentials)
        }
    }

    /// Log into the account
    pub async fn login_with_username(
        &self,
        credentials: dto::request::LoginWithUsername,
    ) -> ServiceResult<entity::AccountId> {
        let login = if let Some(login) = self
            .repo
            .get_login_by_username(&credentials.username)
            .await?
        {
            login
        } else {
            return Err(ServiceError::InvalidCredentials);
        };

        if check_password(credentials.password_hash, login.password_hash).await {
            Ok(login.id)
        } else {
            Err(ServiceError::InvalidCredentials)
        }
    }

    /// Fetch the account details.
    pub async fn me(&self, id: entity::AccountId) -> ServiceResult<dto::response::Account> {
        let (username, username_aliases, email, secondary_emails, keys) = tokio::try_join!(
            self.repo.get_primary_username(id),
            self.repo.get_nonexpiring_username_aliases(id),
            self.repo.get_primary_email(id),
            self.repo.get_secondary_emails(id),
            self.repo.get_keys(id)
        )?;

        let keys = if let Some(keys) = keys {
            keys
        } else {
            return Err(ServiceError::AccountNotFound);
        };

        Ok(dto::response::Account {
            id: id.0,

            username,
            email,

            username_aliases,
            secondary_emails,

            keys: keys.into(),
        })
    }
}
