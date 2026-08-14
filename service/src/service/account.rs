use super::{ServiceError, ServiceResult};
use crate::{dto, entity, id::AccountId, repo::AccountRepo, util::password};

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

        let id = AccountId::unique();
        let account = entity::Account {
            id,
            client_password_kdf: entity::ClientPasswordKdf::Base64EncodedPbkdf2Sha256 {
                salt: signup_dto.password.pbkdf2_salt.clone(),
                iterations: signup_dto.password.pbkdf2_iterations,
                length: signup_dto.password.pbkdf2_length,
            },
            server_password_hash_algorithm: entity::ServerPasswordHashAlgorithm::Argon2id,
            password_hash: Some(password_hash),

            master_key_kek_kdf: entity::ClientPasswordKdf::None,
            master_key_encryption_algorithm: entity::MasterKeyEncryptionAlgorithm::None,
            encrypted_master_key: None,
        };

        transaction.insert_account(&account).await?;

        transaction.insert_default_flags(id).await?;

        transaction
            .add_username(id, &signup_dto.username, true)
            .await?;

        transaction.commit().await?;

        Ok(id)
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
                id: login_credentails.id,
                is_email_verified: login_credentails.is_email_verified,
            })
        } else {
            Err(ServiceError::InvalidCredentials)
        }
    }

    /// Log into the account
    #[tracing::instrument(skip_all)]
    pub async fn login(
        &self,
        login_dto: &dto::request::Login,
    ) -> ServiceResult<dto::service::Login> {
        let Some(login_credentails) = (match &login_dto.account {
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
            dto::request::AccountIdentifier::Id(_) => todo!("get login credentials by email"),
        }) else {
            return Err(ServiceError::InvalidCredentials);
        };

        self.login_with_credentials(&login_credentails, &login_dto.client_password_hash)
            .await
    }

    /// Account's key derivation functions.
    #[tracing::instrument(skip(self))]
    pub async fn get_kdf(
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
            dto::request::AccountIdentifier::Id(id) => {
                self.repo.get_client_password_kdf_by_id(*id).await?
            }
        }) else {
            return Err(ServiceError::AccountNotFound);
        };

        Ok(dto::response::AccountKdf {
            client_password_kdf,
        })
    }

    /// Fetch the account details.
    #[tracing::instrument(skip(self))]
    pub async fn me(&self, id: AccountId) -> ServiceResult<dto::response::Account> {
        let (username, username_aliases, email, secondary_emails) = tokio::try_join!(
            self.repo.get_primary_username(id),
            self.repo.get_nonexpiring_username_aliases(id),
            self.repo.get_primary_email(id),
            self.repo.get_secondary_emails(id),
        )?;

        Ok(dto::response::Account {
            id,

            username,
            email,

            username_aliases,
            secondary_emails,
        })
    }

    /// Change account's password.
    #[tracing::instrument(skip(self, change_password_dto))]
    pub async fn change_password(
        &self,
        id: AccountId,
        change_password_dto: &dto::request::ChangePassword,
    ) -> ServiceResult<()> {
        let Some(login_credentials) = self.repo.get_login_credentials_by_id(id).await? else {
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
            .change_password(
                id,
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
            Err(ServiceError::CannotDeletePrimaryEmailOrEmailNotFound)
        }
    }

    /// Returns true if the email has been taken by the given account.
    #[tracing::instrument(skip_all, fields(id))]
    pub async fn is_email_taken_by(&self, id: AccountId, email: &str) -> ServiceResult<bool> {
        Ok(self.repo.is_email_taken_by(id, email).await?)
    }

    /// Primary email of the account.
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
