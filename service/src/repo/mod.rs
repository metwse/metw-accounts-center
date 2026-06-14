use crate::{dto, entity, id::AccountId};
use async_trait::async_trait;

mod error;

/// Mock repository implementations.
#[cfg(feature = "mock")]
pub mod mock;

pub use error::RepoError;

/// Repository result type.
pub type RepoResult<T> = Result<T, RepoError>;

/// Persistent account storage.
#[async_trait]
pub trait AccountRepo: Send + Sync {
    /// Begin a new transactional unit.
    async fn begin_transaction(&self) -> RepoResult<Box<dyn AccountRepoTransaction>>;

    /// Get password by email.
    ///
    /// Returns Argon2-hashed password with account id. Higher layers shall do
    /// hash verifications.
    async fn get_login_by_email(&self, email: &str) -> RepoResult<Option<dto::repo::Login>>;

    /// Get password by username.
    ///
    /// The behavior is exactly the same with `get_login_by_email`.
    async fn get_login_by_username(&self, username: &str) -> RepoResult<Option<dto::repo::Login>>;

    /// Get primary username if, exists.
    ///
    /// Usually all the accounts have primary usernames, but pending deletion
    /// might drop usernames of an account.
    async fn get_primary_username(&self, id: AccountId) -> RepoResult<Option<String>>;

    /// Get usernames by account id.
    ///
    /// After username change, previous usernames for will continue to belong
    /// the account for a while; but they will be garbage collected.
    async fn get_nonexpiring_username_aliases(&self, id: AccountId) -> RepoResult<Vec<String>>;

    /// Get primary email if, exists.
    ///
    /// All regular accounts shall have primary mail, but some system accounts
    /// or deleted accounts do not.
    async fn get_primary_email(&self, id: AccountId) -> RepoResult<Option<String>>;

    /// Get secondary emails by account id.
    ///
    /// Returns an empty list if none have been added.
    async fn get_secondary_emails(&self, id: AccountId) -> RepoResult<Vec<String>>;

    /// Get account keys - the key bundle of the account.
    async fn get_keys(&self, id: AccountId) -> RepoResult<Option<dto::repo::Keys>>;

    /// Get account flags.
    async fn get_account_flags(&self, id: AccountId) -> RepoResult<Option<entity::AccountFlags>>;

    /// Set the email primary for the account.
    ///
    /// Although `email`s are unique, the `id` parameter is also required to
    /// prevent race conditions. It is highly unlikely that the owner of an
    /// email would change at the exact moment their email is being set as
    /// the primary email, but it is still a safety hazard that must still be
    /// prevented.
    async fn set_primary_email_if_current_is(
        &self,
        id: AccountId,
        current_primary_email: &str,
        new_primary_email: &str,
    ) -> RepoResult<()>;

    /// Remove the email if it is not primary mail of the account.
    async fn remove_email_if_not_primary(&self, id: AccountId, email: &str) -> RepoResult<()>;

    /// Returns true if the username has been taken.
    async fn is_username_taken(&self, username: &str) -> RepoResult<bool>;

    /// Returns true if the email has been taken.
    async fn is_email_taken(&self, email: &str) -> RepoResult<bool>;

    /// Returns true if the email has been taken by the given account.
    async fn is_email_taken_by(&self, id: AccountId, email: &str) -> RepoResult<bool>;
}

/// Transactional repository access wrapper.
#[async_trait]
pub trait AccountRepoTransaction: Send + Sync {
    /// Commit the changes.
    async fn commit(self: Box<Self>) -> RepoResult<()>;

    /// Register a new account, or update its keys.
    async fn upsert_account(
        &mut self,
        id: AccountId,
        password_hash: &str,
        keys: &dto::repo::Keys,
    ) -> RepoResult<()>;

    /// Load default flags to user.
    async fn insert_default_flags(&mut self, id: AccountId) -> RepoResult<()>;

    /// Add a secondary email to the account.
    async fn add_email(&mut self, id: AccountId, email: &str, is_primary: bool) -> RepoResult<()>;

    /// Add username alias to the account.
    async fn add_username(
        &mut self,
        id: AccountId,
        username: &str,
        is_primary: bool,
    ) -> RepoResult<()>;

    /// Set the verified flag of account.
    async fn set_verified_flag(&mut self, id: AccountId, is_verified: bool) -> RepoResult<()>;
}

/// Token provider holds data temporarily.
#[async_trait]
pub trait TokenRepo: Send + Sync {
    /// Atomic operation for checking the revocation and doing it.
    ///
    /// Returns false if the token has already been revoked.
    async fn check_and_revoke(
        &self,
        fingerprint: &[u8],
        revoke_for: std::time::Duration,
    ) -> RepoResult<bool>;

    /// Returns true if the token has been revoked.
    async fn check_revocation(&self, fingerprint: &[u8]) -> RepoResult<bool>;
}
