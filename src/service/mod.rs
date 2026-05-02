use crate::{dto, entity, repo::AccountRepo, snowflake};
use std::sync::Arc;

mod error;

#[cfg(test)]
mod tests;

pub use error::ServiceError;

/// Service result type.
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Account state.
pub struct AccountService {
    repo: Arc<dyn AccountRepo>,
}

impl AccountService {
    /// Creates a new account service.
    pub fn new(repo: Box<dyn AccountRepo>) -> Self {
        Self { repo: repo.into() }
    }

    /// Signup a new account
    pub async fn signup(
        &self,
        signup_dto: dto::request::Signup,
    ) -> ServiceResult<entity::AccountId> {
        let mut transaction = self.repo.begin_transaction().await?;

        let keys = dto::repo::Keys {
            identity_key: signup_dto.keys.identity_key,
            encrypted_private_key: signup_dto.keys.encrypted_private_key,
            encrypted_master_key: signup_dto.keys.encrypted_master_key,
        };

        let account_id = entity::AccountId(snowflake());

        transaction
            .upsert_account(account_id, &signup_dto.password_hash, &keys)
            .await?;

        if !transaction
            .add_username(account_id, &signup_dto.username)
            .await?
        {
            return Err(ServiceError::UsernameTaken);
        }

        if !transaction
            .set_primary_username(&signup_dto.username, true)
            .await?
        {
            // I cannot imagine in which conditions this branch is executed.
            // Most probably this is unreachable.
            return Err(ServiceError::UnexceptedError("could not take username"));
        }

        // TODO: send verification emails

        transaction.commit().await?;

        Ok(account_id)
    }
}
