use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Account id
#[derive(Serialize, Deserialize, FromRow, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct AccountId(pub i64);

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Account entity.
///
/// This type mainly used for storing cryptographic primitives.
#[derive(FromRow, Default)]
pub struct Account {
    /// Account id generated using Twitter's snowflake algorithm.
    pub id: AccountId,

    /// Argon2 hashed password.
    pub password_hash: String,

    /// Identity public key in DER format.
    pub identity_key: Vec<u8>,

    /// Private key encrypted by the master key.
    pub encrpyted_private_key: Vec<u8>,

    /// Master key encrypted by user's password.
    pub encrpyted_master_key: Vec<u8>,
}

/// Account flags entity.
#[derive(FromRow, Clone)]
pub struct AccountFlags {
    /// Accounts associated with the flags entity.
    pub id: AccountId,

    /// Whether or not the account has been verified.
    pub is_verified: bool,
}

/// Usernames or username aliases assigned to an account.
#[derive(FromRow, Default)]
pub struct Username {
    /// Username.
    pub username: String,

    /// Account have the username.
    pub account_id: AccountId,

    /// Whether or not the username is account's primary username.
    pub is_primary: bool,

    /// Timestamp the username taken at.
    pub created_at: DateTime<Utc>,

    /// Timestamp the username expires at.
    pub expires_at: Option<DateTime<Utc>>,
}

/// Verified email of an account.
#[derive(FromRow, Default)]
pub struct Email {
    /// Email.
    pub email: String,

    /// Account have the username.
    pub account_id: AccountId,

    /// Whether or not the email is primary mail of the account.
    pub is_primary: bool,

    /// Timestamp the username expires at.
    pub created_at: DateTime<Utc>,
}
