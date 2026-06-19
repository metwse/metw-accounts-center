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
    ///
    /// Integrity Note: Emails are unique, as defined in [`I-AR-5`] and
    /// [`I-AR-8`]. There may be more than one AddEmail token for the same
    /// address, but only the first evaluated token will be successful.
    ///
    /// [`I-AR-5`]: crate::repo::AccountRepo#invariants
    /// [`I-AR-8`]: crate::repo::AccountRepo#invariants
    AddEmail { email: String },

    /// Allow changing account's primary email address to given address.
    ///
    /// Integrity Note: An account can have at most one primary email, as
    /// defined in [`I-AR-3`]. There may be more than one ChangePrimaryEmail
    /// token, but only the token that changes the primary email from the
    /// current one to the new one will be successful.
    ///
    /// [`I-AR-3`]: crate::repo::AccountRepo#invariants
    ChangePrimaryEmail {
        current_primary_email: String,
        new_primary_email: String,
    },

    /// Enable account and add first primary email.
    ///
    /// Integrity Note: An account can have at most one primary email, as
    /// defined in [`I-AR-3`]. More than one CompleteSignup token may be valid
    /// at any time, but only one of them will be successful based on the
    /// invariant.
    ///
    /// [`I-AR-3`]: crate::repo::AccountRepo#invariants
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
            Self::Session => Duration::from_hours(24 * 7),
            Self::PendingActivationSession { .. } => Duration::from_hours(1),
            Self::AddEmail { .. } => Duration::from_hours(1),
            Self::ChangePrimaryEmail { .. } => Duration::from_mins(10),
            Self::CompleteSignup { .. } => Duration::from_hours(1),
        }
    }
}
