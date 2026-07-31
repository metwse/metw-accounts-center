use crate::id::AccountId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Key derivation algorithm applied by the server to the account's password.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "algorithm", rename_all = "snake_case")]
pub enum ServerPasswordHashAlgorithm {
    /// No additional server-side hashing is applied.
    None,
    /// Argon2id
    Argon2id,
}

/// Key derivation applied client-side.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "algorithm", rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum ClientPasswordKdf {
    /// Plain text.
    None,
    /// For legacy accounts.
    LegacySha256Hex,
    /// Base64-encoded PBKDF2-SHA256.
    Base64EncodedPbkdf2Sha256 {
        salt: String,
        iterations: u32,
        length: u32,
    },
}

/// Algorithm used for encrypting the account master key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "algorithm", rename_all = "snake_case")]
pub enum MasterKeyEncryptionAlgorithm {
    /// Master-key encryption is not applied.
    None,
    /// ChaCha20-Poly1305.
    ChaCha20Poly1305,
}

/// Account entity.
///
/// This type mainly used for storing cryptographic primitives.
#[derive(Debug, Clone)]
pub struct Account {
    /// Account ID generated using Twitter's snowflake algorithm.
    pub id: AccountId,

    /// KDF for login.
    pub client_password_kdf: ClientPasswordKdf,

    /// Hash function applied before saving the password.
    pub server_password_hash_algorithm: ServerPasswordHashAlgorithm,

    /// Server-side password representation.
    ///
    /// Absent for accounts that can no longer authenticate, such as deleted
    /// accounts.
    pub password_hash: Option<String>,

    /// KDF for producing the Key Encryption Key (KEK) for master key.
    pub master_key_kek_kdf: ClientPasswordKdf,

    /// Algorithm client used to encrypt its master key.
    pub master_key_encryption_algorithm: MasterKeyEncryptionAlgorithm,

    /// Client-provided encrypted master key.
    ///
    /// Absent until a master key is assigned to the account.
    pub encrypted_master_key: Option<Vec<u8>>,
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

    /// Whether or not the email is primary email of the account.
    pub is_primary: bool,

    /// Timestamp the email was added at.
    pub created_at: DateTime<Utc>,
}
