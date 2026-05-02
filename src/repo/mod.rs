use crate::{dto, entity};
use async_trait::async_trait;

/// Persistent account storage.
#[async_trait]
pub trait AccountRepo {
    /// Register a new account, or update its keys.
    async fn upsert_account(&self, id: entity::AccountId, password_hash: &str, keys: &dto::repo::Keys);

    /// Get password by email.
    ///
    /// Returns Argon2-hashed password with account id. Higher layers shall do
    /// hash verifications.
    async fn get_login_by_email(&self, email: &str) -> Option<dto::repo::Login>;

    /// Get password by username.
    ///
    /// The behavior is exactly the same with `get_login_by_email`.
    async fn get_login_by_username(&self, username: &str) -> Option<dto::repo::Login>;

    /// Get primary username if, exists.
    ///
    /// Usually all the accounts have primary usernames, but pending deletion
    /// might drop usernames of an account.
    async fn get_primary_username(&self, id: entity::AccountId) -> Option<String>;

    /// Get usernames by account id.
    ///
    /// After username change, previous usernames for will continue to belong
    /// the account for a while; but they will be garbage collected.
    async fn get_nonexpiring_username_aliases(&self, id: entity::AccountId) -> Vec<String>;

    /// Get primary email if, exists.
    ///
    /// All regular accounts shall have primary mail, but some system accounts
    /// or deleted accounts do not.
    async fn get_primary_email(&self, id: entity::AccountId) -> Option<String>;

    /// Get secondary by account id.
    ///
    /// List of secondary emails if user add.
    async fn get_secondary_emails(&self, id: entity::AccountId) -> Vec<String>;

    /// Get account keys - the key bundle of the account.
    async fn get_keys(&self, id: entity::AccountId) -> Option<dto::repo::Keys>;

    /// Add a secondary email to the account. Returns true if the operation
    /// succeed.
    async fn add_email(&self, id: entity::AccountId, email: &str) -> bool;

    /// Add username alias to the account. Returns true if the operation
    /// succeed.
    async fn add_username(&self, id: entity::AccountId, username: &str) -> bool;

    /// Set the email primary for the account.
    async fn set_primary_email(&self, id: entity::AccountId, email: &str) -> bool;

    /// Set the username primary for the account.
    async fn set_primary_username(&self, id: entity::AccountId, username: &str) -> bool;
}

/// Token provider holds data temporarily.
#[async_trait]
pub trait TokenRepo {}
