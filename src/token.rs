use crate::id::AccountId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Authentication token with authorization scopes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    /// Account id.
    pub id: AccountId,
    /// Token's persmissions.
    pub scope: TokenScope,

    pub(crate) exp: usize,
    nbf: usize,
    iat: usize,
}

/// Authorization scopes.
#[derive(Debug, Serialize, Deserialize)]
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
        let iat = Utc::now().timestamp() as usize;
        let exp = iat + (valid_for.as_secs() as usize);

        Token {
            id,
            scope,
            exp,
            nbf: iat,
            iat,
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
