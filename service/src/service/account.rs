use super::{ServiceError, ServiceResult};
use crate::{dto, id::AccountId, repo::AccountRepo, util::password};

/// Account state.
pub struct AccountService {
    repo: Box<dyn AccountRepo>,
}

impl AccountService {
    /// Creates a new account service.
    pub fn new(repo: Box<dyn AccountRepo>) -> Self {
        Self { repo }
    }

    /// Sign up a new account.
    #[tracing::instrument(skip_all)]
    pub async fn signup(&self, signup_dto: &dto::request::Signup) -> ServiceResult<AccountId> {
        if self.repo.is_username_taken(&signup_dto.username).await? {
            return Err(ServiceError::UsernameTaken);
        }

        if self.repo.is_email_taken(&signup_dto.email).await? {
            return Err(ServiceError::EmailTaken);
        }

        let password_hash = password::hash(&signup_dto.client_password_hash).await;

        let mut transaction = self.repo.begin_transaction().await?;

        let keys = dto::repo::Keys {
            identity_key: &signup_dto.keys.identity_key,
            encrypted_private_key: &signup_dto.keys.encrypted_private_key,
            encrypted_master_key: &signup_dto.keys.encrypted_master_key,
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
        login_credentails: &dto::repo::OwnedLoginCredentials,
        client_password_hash: &str,
    ) -> ServiceResult<dto::service::Login> {
        if password::check(client_password_hash, &login_credentails.password_hash).await {
            Ok(dto::service::Login {
                id: login_credentails.id,
                is_email_verified: login_credentails.is_email_verified,
            })
        } else {
            Err(ServiceError::InvalidCredentials)
        }
    }

    /// Log into the account
    #[tracing::instrument(skip_all)]
    pub async fn login_with_email(
        &self,
        credentials: &dto::request::LoginWithEmail,
    ) -> ServiceResult<dto::service::Login> {
        let Some(login_credentails) = self
            .repo
            .get_login_credentials_by_email(&credentials.email)
            .await?
        else {
            return Err(ServiceError::InvalidCredentials);
        };

        self.login(&login_credentails, &credentials.client_password_hash)
            .await
    }

    /// Log into the account
    #[tracing::instrument(skip_all)]
    pub async fn login_with_username(
        &self,
        credentials: &dto::request::LoginWithUsername,
    ) -> ServiceResult<dto::service::Login> {
        let Some(login) = self
            .repo
            .get_login_credentials_by_username(&credentials.username)
            .await?
        else {
            return Err(ServiceError::InvalidCredentials);
        };

        self.login(&login, &credentials.client_password_hash).await
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
    pub async fn is_username_taken(&self, username: &str) -> ServiceResult<bool> {
        Ok(self.repo.is_username_taken(username).await?)
    }

    /// Wheter or not the email has been taken.
    pub async fn is_email_taken(&self, email: &str) -> ServiceResult<bool> {
        Ok(self.repo.is_email_taken(email).await?)
    }

    /// Remove a secondary email.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn remove_email_if_not_primary(
        &self,
        id: AccountId,
        email: &str,
    ) -> ServiceResult<()> {
        if self.repo.remove_email_if_not_primary(id, email).await? {
            Ok(())
        } else {
            Err(ServiceError::CannotDeletePrimaryEmail)
        }
    }

    /// Returns true if the email has been taken by the given account.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn is_email_taken_by(&self, id: AccountId, email: &str) -> ServiceResult<bool> {
        Ok(self.repo.is_email_taken_by(id, email).await?)
    }

    /// Primary mail of the account.
    #[tracing::instrument(skip(self))]
    pub async fn get_primary_email(&self, id: AccountId) -> ServiceResult<Option<String>> {
        Ok(self.repo.get_primary_email(id).await?)
    }

    /// Primary username of the account.
    #[tracing::instrument(skip(self))]
    pub async fn get_primary_username(&self, id: AccountId) -> ServiceResult<Option<String>> {
        Ok(self.repo.get_primary_username(id).await?)
    }

    /// Add the email as a secondary email to the account.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn auth_add_email(&self, id: AccountId, email: &str) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .add_email(id, email, false)
            .await
            .map_err(|_| ServiceError::AddEmailFailed)?;
        transaction.commit().await?;

        Ok(())
    }

    /// Change account's primary email.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn auth_change_primary_email(
        &self,
        id: AccountId,
        current_primary_email: &str,
        new_primary_email: &str,
    ) -> ServiceResult<()> {
        if self
            .repo
            .set_primary_email_if_current_is(id, current_primary_email, new_primary_email)
            .await?
        {
            Ok(())
        } else {
            Err(ServiceError::ChangePrimaryEmailFailed)
        }
    }

    /// Complete signup by adding the email and activating the account.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn auth_complete_signup(&self, id: AccountId, email: &str) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .add_email(id, email, true)
            .await
            .map_err(|_| ServiceError::SignupCompleteFailed)?;
        transaction.set_is_email_verified_flag(id, true).await?;
        transaction.commit().await?;

        Ok(())
    }
}
