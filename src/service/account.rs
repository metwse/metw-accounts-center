use super::{ServiceError, ServiceResult};
use crate::{dto, id::AccountId, repo::AccountRepo, util::password};

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
    #[tracing::instrument(skip_all)]
    pub async fn signup(&self, signup_dto: dto::request::Signup) -> ServiceResult<AccountId> {
        if self.repo.is_username_taken(&signup_dto.username).await? {
            return Err(ServiceError::UsernameTaken);
        }

        if self.repo.is_email_taken(&signup_dto.email).await? {
            return Err(ServiceError::EmailTaken);
        }

        let password_hash = password::hash(signup_dto.password_hash).await;

        let mut transaction = self.repo.begin_transaction().await?;

        let keys = dto::repo::Keys {
            identity_key: signup_dto.keys.identity_key,
            encrypted_private_key: signup_dto.keys.encrypted_private_key,
            encrypted_master_key: signup_dto.keys.encrypted_master_key,
        };

        let id = AccountId::unique();

        transaction
            .upsert_account(id, &password_hash, &keys)
            .await?;

        transaction.insert_default_flags(id).await?;

        transaction
            .add_username(id, &signup_dto.username, true)
            .await?;

        transaction.commit().await?;

        Ok(id)
    }

    /// For use with login.
    async fn login(
        &self,
        password_hash: String,
        login: dto::repo::Login,
    ) -> ServiceResult<AccountId> {
        if let Some(flags) = self.repo.get_account_flags(login.id).await?
            && flags.is_verified
        {
            if password::check(password_hash, login.password_hash).await {
                Ok(login.id)
            } else {
                Err(ServiceError::InvalidCredentials)
            }
        } else {
            Err(ServiceError::AccountNotVerified)
        }
    }

    /// Log into the account
    #[tracing::instrument(skip_all)]
    pub async fn login_with_email(
        &self,
        credentials: dto::request::LoginWithEmail,
    ) -> ServiceResult<AccountId> {
        let Some(login) = self.repo.get_login_by_email(&credentials.email).await? else {
            return Err(ServiceError::InvalidCredentials);
        };

        self.login(credentials.password_hash, login).await
    }

    /// Log into the account
    #[tracing::instrument(skip_all)]
    pub async fn login_with_username(
        &self,
        credentials: dto::request::LoginWithUsername,
    ) -> ServiceResult<AccountId> {
        let Some(login) = self
            .repo
            .get_login_by_username(&credentials.username)
            .await?
        else {
            return Err(ServiceError::InvalidCredentials);
        };

        self.login(credentials.password_hash, login).await
    }

    /// Fetch the account details.
    #[tracing::instrument(skip(self))]
    pub async fn me(&self, id: AccountId) -> ServiceResult<dto::response::Account> {
        let (username, username_aliases, email, secondary_emails, keys) = tokio::try_join!(
            self.repo.get_primary_username(id),
            self.repo.get_nonexpiring_username_aliases(id),
            self.repo.get_primary_email(id),
            self.repo.get_secondary_emails(id),
            self.repo.get_keys(id)
        )?;

        let Some(keys) = keys else {
            return Err(ServiceError::AccountNotFound);
        };

        Ok(dto::response::Account {
            id: id.into(),

            username,
            email,

            username_aliases,
            secondary_emails,

            keys: keys.into(),
        })
    }

    /// Wheter or not the username has been taken.
    pub async fn is_username_taken(&self, username: String) -> ServiceResult<bool> {
        Ok(self.repo.is_username_taken(&username).await?)
    }

    /// Wheter or not the email has been taken.
    pub async fn is_email_taken(&self, email: String) -> ServiceResult<bool> {
        Ok(self.repo.is_email_taken(&email).await?)
    }

    /// Add the email as a secondary email to the account.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn auth_add_email(&self, id: AccountId, email: String) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction.add_email(id, &email, false).await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Remove a secondary email.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn remove_email_if_not_primary(
        &self,
        id: AccountId,
        email: String,
    ) -> ServiceResult<()> {
        Ok(self.repo.remove_email_if_not_primary(id, &email).await?)
    }

    /// Returns true if the email has been taken by the given account.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn is_email_taken_by(&self, id: AccountId, email: String) -> ServiceResult<bool> {
        Ok(self.repo.is_email_taken_by(id, &email).await?)
    }

    /// Primary mail of the account.
    #[tracing::instrument(skip(self))]
    pub async fn get_primary_email(&self, id: AccountId) -> ServiceResult<Option<String>> {
        Ok(self.repo.get_primary_email(id).await?)
    }

    /// Change account's primary email.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn auth_change_primary_email(
        &self,
        id: AccountId,
        current_primary_email: String,
        new_primary_email: String,
    ) -> ServiceResult<()> {
        self.repo
            .set_primary_email_if_current_is(id, &current_primary_email, &new_primary_email)
            .await?;

        Ok(())
    }

    /// Complete signup by adding the email and activating the account.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn auth_complete_signup(&self, id: AccountId, email: String) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction.add_email(id, &email, true).await?;
        transaction.set_verified_flag(id, true).await?;
        transaction.commit().await?;

        Ok(())
    }
}
