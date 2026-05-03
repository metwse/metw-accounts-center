use crate::{dto, entity};
use async_trait::async_trait;

mod error;

/// Repository implementations.
pub mod impls;

pub use error::RepoError;

/// Repository result type.
pub type RepoResult<T> = Result<T, RepoError>;

/// Persistent account storage.
#[async_trait]
pub trait AccountRepo {
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
    async fn get_primary_username(&self, id: entity::AccountId) -> RepoResult<Option<String>>;

    /// Get usernames by account id.
    ///
    /// After username change, previous usernames for will continue to belong
    /// the account for a while; but they will be garbage collected.
    async fn get_nonexpiring_username_aliases(
        &self,
        id: entity::AccountId,
    ) -> RepoResult<Vec<String>>;

    /// Get primary email if, exists.
    ///
    /// All regular accounts shall have primary mail, but some system accounts
    /// or deleted accounts do not.
    async fn get_primary_email(&self, id: entity::AccountId) -> RepoResult<Option<String>>;

    /// Get secondary by account id.
    ///
    /// List of secondary emails if user add.
    async fn get_secondary_emails(&self, id: entity::AccountId) -> RepoResult<Vec<String>>;

    /// Get account keys - the key bundle of the account.
    async fn get_keys(&self, id: entity::AccountId) -> RepoResult<Option<dto::repo::Keys>>;
}

/// Transactional repository access wrapper.
#[async_trait]
pub trait AccountRepoTransaction {
    /// Commit the changes.
    async fn commit(self: Box<Self>) -> RepoResult<()>;

    /// Register a new account, or update its keys.
    async fn upsert_account(
        &mut self,
        id: entity::AccountId,
        password_hash: &str,
        keys: &dto::repo::Keys,
    ) -> RepoResult<()>;

    /// Add a secondary email to the account. Returns true if the operation
    /// succeed.
    async fn add_email(&mut self, id: entity::AccountId, email: &str) -> RepoResult<bool>;

    /// Add username alias to the account. Returns true if the operation
    /// succeed.
    async fn add_username(&mut self, id: entity::AccountId, username: &str) -> RepoResult<bool>;

    /// Set the email primary for the account.
    ///
    /// Although `email`s are unique, the `id` parameter is also required to
    /// prevent race conditions. It is highly unlikely that the owner of an
    /// email would change at the exact moment the one's email is set as the
    /// primary key, but it is still an unsafety that must still be prevented.
    async fn set_primary_email(
        &mut self,
        id: entity::AccountId,
        email: &str,
        is_primary: bool,
    ) -> RepoResult<bool>;

    /// Set the username primary for the account.
    ///
    /// See [`AccountRepoTransaction::set_primary_email`]
    async fn set_primary_username(
        &mut self,
        id: entity::AccountId,
        username: &str,
        is_primary: bool,
    ) -> RepoResult<bool>;
}

/// Token provider holds data temporarily.
#[async_trait]
pub trait TokenRepo {
    /// Revoke the token with provided fingerprint. Keep the fingerprint for
    /// at least `revoke_for` time.
    async fn revoke(&self, fingerprint: &[u8], revoke_for: std::time::Duration) -> RepoResult<()>;

    /// Returns true if the token has been revoked.
    async fn check_revocation(&self, fingerprint: &[u8]) -> RepoResult<bool>;
}
