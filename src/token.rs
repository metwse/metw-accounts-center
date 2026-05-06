use crate::id::AccountId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Authentication token with authorization scopes.
#[derive(Debug)]
pub struct Token {
    /// Account id.
    pub id: AccountId,
    /// Token's permissions.
    pub scope: TokenScope,
    /// Duration the token is valid for.
    pub valid_for: Duration,
}

/// Authorization scopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum TokenScope {
    /// Permit logins.
    Authenticate,

    /// Permit adding the email to the account.
    AddEmail(String),
    /// Allow changing account's primary email address to given address.
    SetPrimaryEmail {
        current_primary_email: String,
        new_primary_email: String,
    },
    /// Enable account and add first primary email. This scope is present in
    /// email sent in signup procedure.
    Signup { email: String },
}

impl Token {
    /// Create a new token.
    pub fn new(id: AccountId, scope: TokenScope, valid_for: Duration) -> Self {
        Token {
            id,
            scope,
            valid_for,
        }
    }
}

impl TokenScope {
    /// Get name of the enum variant.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Authenticate => "authenticate",
            Self::AddEmail(..) => "add-email",
            Self::SetPrimaryEmail { .. } => "set-primary-email",
            Self::Signup { .. } => "signup",
        }
    }
}
