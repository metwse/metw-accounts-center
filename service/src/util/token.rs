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
    pub lifetime: Duration,
}

/// Authorization scopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum TokenScope {
    /// Permit logins.
    Session,
    /// Retry signup session.
    PendingActivationSession,

    /// The email to the account.
    AddEmail { email: String },
    /// Allow changing account's primary email address to given address.
    ChangePrimaryEmail {
        current_primary_email: String,
        new_primary_email: String,
    },
    /// Enable account and add first primary email. This scope is present in
    /// email sent in signup procedure.
    CompleteSignup { email: String },
}

impl Token {
    /// Create a new token.
    pub fn new(id: AccountId, scope: TokenScope) -> Self {
        let lifetime = scope.lifetime();

        Self {
            id,
            scope,
            lifetime,
        }
    }

    /// Create a new token.
    pub fn new_with_lifetime(id: AccountId, scope: TokenScope, lifetime: Duration) -> Self {
        Self {
            id,
            scope,
            lifetime,
        }
    }
}

impl TokenScope {
    /// Get name of the enum variant.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::PendingActivationSession { .. } => "pending-activation-session",
            Self::AddEmail { .. } => "add-email",
            Self::ChangePrimaryEmail { .. } => "change-primary-email",
            Self::CompleteSignup { .. } => "complete-signup",
        }
    }

    /// Duration the token type valid for.
    pub fn lifetime(&self) -> Duration {
        match self {
            Self::Session => Duration::from_secs(60 * 60 * 24 * 7),
            Self::PendingActivationSession { .. } => Duration::from_secs(1),
            Self::AddEmail { .. } => Duration::from_secs(60 * 60),
            Self::ChangePrimaryEmail { .. } => Duration::from_secs(60 * 10),
            Self::CompleteSignup { .. } => Duration::from_secs(60 * 60 * 24),
        }
    }
}
