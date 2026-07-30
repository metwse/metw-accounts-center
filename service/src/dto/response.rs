use crate::id::AccountId;
use serde::Serialize;
use utoipa::ToSchema;

/// Account entity returned to user after authentication.
#[derive(Debug, Serialize, ToSchema)]
pub struct Account {
    /// Unique user id.
    pub id: AccountId,

    /// User's primary username, if exists.
    pub username: Option<String>,
    /// Primary email address, if exitsts.
    pub email: Option<String>,

    /// Non-expiring username aliases.
    pub username_aliases: Vec<String>,
    /// Secondary emails.
    pub secondary_emails: Vec<String>,
}

/// Base64-encoded, usually returned after sign up or log in.
#[derive(Debug, Serialize, ToSchema)]
pub struct Token {
    pub token: String,
}
