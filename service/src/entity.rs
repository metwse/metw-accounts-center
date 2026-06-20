use crate::id::AccountId;
use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

/// Account entity.
///
/// This type mainly used for storing cryptographic primitives.
#[derive(Debug, FromRow, Default)]
pub struct Account {
    /// Account ID generated using Twitter's snowflake algorithm.
    pub id: AccountId,

    /// Argon2 hashed password.
    pub password_hash: String,

    /// Identity public key in DER format.
    pub identity_key: Vec<u8>,

    /// Private key encrypted by the master key.
    pub encrypted_private_key: Vec<u8>,

    /// Master key encrypted by user's password.
    pub encrypted_master_key: Vec<u8>,
}

/// Account flags entity.
#[derive(Debug, FromRow, Clone)]
pub struct AccountFlags {
    /// Accounts associated with the flags entity.
    pub id: AccountId,

    /// Whether or not the account has been verified.
    pub is_email_verified: bool,
}

/// Usernames or username aliases assigned to an account.
#[derive(Debug, FromRow, Default)]
pub struct Username {
    /// Username.
    pub username: String,

    /// Account the username belongs to.
    pub account_id: AccountId,

    /// Whether or not the username is account's primary username.
    pub is_primary: bool,

    /// Timestamp the username was taken.
    pub created_at: DateTime<Utc>,

    /// Timestamp the username expires at.
    pub expires_at: Option<DateTime<Utc>>,
}

/// Verified email of an account.
#[derive(Debug, FromRow, Default)]
pub struct Email {
    /// Email.
    pub email: String,

    /// Account the email belongs to.
    pub account_id: AccountId,

    /// Whether or not the email is primary mail of the account.
    pub is_primary: bool,

    /// Timestamp the email was added at.
    pub created_at: DateTime<Utc>,
}
