use super::{ServiceError, ServiceResult};
use crate::{
    dto, entity,
    id::AccountId,
    repo::{AccountRepo, limits},
    util::password,
};

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
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(account_id = tracing::field::Empty, username = signup_dto.username)
    )]
    pub async fn signup(&self, signup_dto: &dto::request::Signup) -> ServiceResult<AccountId> {
        let (is_email_taken_res, is_username_taken_rs) = tokio::join!(
            self.repo.is_email_taken(&signup_dto.email),
            self.repo.is_username_taken(&signup_dto.username)
        );

        if is_email_taken_res? {
            return Err(ServiceError::EmailTaken);
        }

        if is_username_taken_rs? {
            return Err(ServiceError::UsernameTaken);
        }

        let password_hash = password::hash(&signup_dto.password.base64_hash).await;

        let mut transaction = self.repo.begin_transaction().await?;

        let account_id = AccountId::unique();
        let account = entity::Account {
            account_id,
            client_password_kdf: entity::ClientPasswordKdf::Base64EncodedPbkdf2Sha256 {
                salt: signup_dto.password.pbkdf2_salt.clone(),
                iterations: signup_dto.password.pbkdf2_iterations,
                length: signup_dto.password.pbkdf2_length,
            },
            server_password_hash_algorithm: entity::ServerPasswordHashAlgorithm::Argon2id,
            password_hash: Some(password_hash),

            master_key_kek_kdf: entity::ClientPasswordKdf::None,
            master_key_encryption_algorithm: entity::KeyEncryptionAlgorithm::None,
            encrypted_master_key: None,
            master_key_id: None,
        };

        transaction.insert(&account).await?;

        transaction.insert_default_flags(account_id).await?;

        transaction
            .insert_username(account_id, &signup_dto.username, true)
            .await?;

        transaction.commit().await?;

        tracing::Span::current().record("account_id", account_id.to_string());

        Ok(account_id)
    }

    /// For use with login.
    async fn login_with_credentials(
        &self,
        login_credentails: &dto::repo::OwnedLoginCredentials,
        client_password_hash: &str,
    ) -> ServiceResult<dto::service::Login> {
        let Some(password_hash) = &login_credentails.password_hash else {
            return Err(ServiceError::InvalidCredentials);
        };

        if match login_credentails.server_password_hash_algorithm {
            entity::ServerPasswordHashAlgorithm::None if client_password_hash == password_hash => {
                true
            }

            entity::ServerPasswordHashAlgorithm::Argon2id
                if password::check(client_password_hash, password_hash).await =>
            {
                true
            }

            _ => false,
        } {
            Ok(dto::service::Login {
                account_id: login_credentails.account_id,
                is_email_verified: login_credentails.is_email_verified,
            })
        } else {
            Err(ServiceError::InvalidCredentials)
        }
    }

    /// Log into the account
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(account_identifier = ?login_dto.account_identifier)
    )]
    pub async fn verify_login(
        &self,
        login_dto: &dto::request::Login,
    ) -> ServiceResult<dto::service::Login> {
        let Some(login_credentails) = (match &login_dto.account_identifier {
            dto::request::AccountIdentifier::Username(username) => {
                self.repo
                    .get_login_credentials_by_username(&username.username)
                    .await?
            }
            dto::request::AccountIdentifier::Email(email) => {
                self.repo
                    .get_login_credentials_by_email(&email.email)
                    .await?
            }
            dto::request::AccountIdentifier::AccountId(account_id) => {
                self.repo
                    .get_login_credentials_by_account_id(*account_id)
                    .await?
            }
        }) else {
            return Err(ServiceError::AccountNotFound);
        };

        self.login_with_credentials(&login_credentails, &login_dto.client_password_hash)
            .await
    }

    /// Account's key derivation functions.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_client_password_kdf(
        &self,
        account_identifier: &dto::request::AccountIdentifier,
    ) -> ServiceResult<dto::response::AccountKdf> {
        let Some(client_password_kdf) = (match account_identifier {
            dto::request::AccountIdentifier::Username(username) => {
                self.repo
                    .get_client_password_kdf_by_username(&username.username)
                    .await?
            }
            dto::request::AccountIdentifier::Email(email) => {
                self.repo
                    .get_client_password_kdf_by_email(&email.email)
                    .await?
            }
            dto::request::AccountIdentifier::AccountId(account_id) => {
                self.repo
                    .get_client_password_kdf_by_account_id(*account_id)
                    .await?
            }
        }) else {
            return Err(ServiceError::AccountNotFound);
        };

        Ok(dto::response::AccountKdf {
            client_password_kdf,
        })
    }

    /// Fetch the account details.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get(&self, account_id: AccountId) -> ServiceResult<dto::response::Account> {
        let (username, username_aliases, email, secondary_emails) = tokio::try_join!(
            self.repo.get_primary_username(account_id),
            self.repo.get_nonexpiring_username_aliases(account_id),
            self.repo.get_primary_email(account_id),
            self.repo.get_secondary_emails(account_id),
        )?;

        Ok(dto::response::Account {
            account_id,

            username,
            email,

            username_aliases,
            secondary_emails,
        })
    }

    /// Change account's password.
    #[tracing::instrument(level = "debug", skip(self, change_password_dto))]
    pub async fn change_password(
        &self,
        account_id: AccountId,
        change_password_dto: &dto::request::ChangePassword,
    ) -> ServiceResult<()> {
        let Some(login_credentials) = self
            .repo
            .get_login_credentials_by_account_id(account_id)
            .await?
        else {
            return Err(ServiceError::AccountNotFound);
        };

        self.login_with_credentials(
            &login_credentials,
            &change_password_dto.current_password_hash,
        )
        .await?;

        let new_password = &change_password_dto.new_password;

        let server_derived = password::hash(&new_password.base64_hash).await;

        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .set_password_credentials(
                account_id,
                &server_derived,
                &entity::ClientPasswordKdf::Base64EncodedPbkdf2Sha256 {
                    salt: new_password.pbkdf2_salt.clone(),
                    iterations: new_password.pbkdf2_iterations,
                    length: new_password.pbkdf2_length,
                },
                &entity::ServerPasswordHashAlgorithm::Argon2id,
            )
            .await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Whether or not the username has been taken.
    pub async fn is_username_taken(&self, username: &str) -> ServiceResult<bool> {
        Ok(self.repo.is_username_taken(username).await?)
    }

    /// Whether or not the email has been taken.
    pub async fn is_email_taken(&self, email: &str) -> ServiceResult<bool> {
        Ok(self.repo.is_email_taken(email).await?)
    }

    /// Remove a secondary email.
    #[tracing::instrument(level = "debug", skip_all, fields(account_id))]
    pub async fn remove_email_if_not_primary(
        &self,
        account_id: AccountId,
        email: &str,
    ) -> ServiceResult<()> {
        if self
            .repo
            .delete_email_if_not_primary(account_id, email)
            .await?
        {
            Ok(())
        } else {
            Err(ServiceError::CannotDeletePrimaryEmailOrEmailNotFound)
        }
    }

    /// Returns true if the email has been taken by the given account.
    pub async fn is_email_owned_by(
        &self,
        account_id: AccountId,
        email: &str,
    ) -> ServiceResult<bool> {
        Ok(self.repo.is_email_owned_by(account_id, email).await?)
    }

    /// Primary email of the account.
    pub async fn get_primary_email(&self, account_id: AccountId) -> ServiceResult<Option<String>> {
        Ok(self.repo.get_primary_email(account_id).await?)
    }

    /// Primary username of the account.
    pub async fn get_primary_username(
        &self,
        account_id: AccountId,
    ) -> ServiceResult<Option<String>> {
        Ok(self.repo.get_primary_username(account_id).await?)
    }

    /// Add the email as a secondary email to the account.
    #[tracing::instrument(level = "debug", skip_all, fields(account_id))]
    pub async fn confirm_email_addition(
        &self,
        account_id: AccountId,
        email: &str,
    ) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;

        transaction.lock(account_id).await?;

        if transaction.count_emails(account_id).await? >= limits::account_repo::MAXIMUM_EMAIL_COUNT
        {
            return Err(ServiceError::TooManyEmails(
                limits::account_repo::MAXIMUM_EMAIL_COUNT,
            ));
        }

        transaction
            .insert_email(account_id, email, false)
            .await
            .map_err(|_| ServiceError::AddEmailFailed)?;

        transaction.commit().await?;

        Ok(())
    }

    /// Change account's primary email.
    #[tracing::instrument(level = "debug", skip_all, fields(account_id))]
    pub async fn confirm_primary_email_change(
        &self,
        account_id: AccountId,
        current_primary_email: &str,
        new_primary_email: &str,
    ) -> ServiceResult<()> {
        if self
            .repo
            .compare_and_set_primary_email(account_id, current_primary_email, new_primary_email)
            .await?
        {
            Ok(())
        } else {
            Err(ServiceError::ChangePrimaryEmailFailed)
        }
    }

    /// Complete signup by adding the email and activating the account.
    #[tracing::instrument(level = "debug", skip_all, fields(account_id))]
    pub async fn complete_signup(&self, account_id: AccountId, email: &str) -> ServiceResult<()> {
        let mut transaction = self.repo.begin_transaction().await?;
        transaction
            .insert_email(account_id, email, true)
            .await
            .map_err(|_| ServiceError::SignupCompleteFailed)?;
        transaction
            .set_email_verified_flag(account_id, true)
            .await?;
        transaction.commit().await?;

        Ok(())
    }
}
