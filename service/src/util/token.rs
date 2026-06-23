use crate::{checked_now, id::AccountId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Authentication token with authorization scopes.
#[derive(Debug)]
pub struct Token {
    /// Account id.
    pub id: AccountId,

    /// Token's permissions.
    pub scope: TokenScope,
}

/// Authentication token with authorization scopes.
#[derive(Debug, Clone)]
pub struct DecodedToken {
    /// Account id.
    pub id: AccountId,

    /// Token's permissions.
    pub scope: TokenScope,

    /// Byte fingerprint unique to the token.
    pub fingerprint: Vec<u8>,

    /// The time token expires at.
    pub expires_at: DateTime<Utc>,

    /// Time token is issued at.
    pub issued_at: DateTime<Utc>,
}

/// Authorization scopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum TokenScope {
    /// Permit logins.
    Session,

    /// Retry signup session.
    EmailVerificationSession,

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

impl TokenScope {
    /// Get the scope name.
    pub fn scope_name(&self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::EmailVerificationSession { .. } => "pending-activation-session",
            Self::AddEmail { .. } => "add-email",
            Self::ChangePrimaryEmail { .. } => "change-primary-email",
            Self::CompleteSignup { .. } => "complete-signup",
        }
    }

    /// Duration the token type valid for.
    pub fn lifetime(&self) -> Duration {
        match self {
            Self::Session => Duration::from_hours(24 * 7),
            Self::EmailVerificationSession { .. } => Duration::from_hours(1),
            Self::AddEmail { .. } => Duration::from_hours(1),
            Self::ChangePrimaryEmail { .. } => Duration::from_mins(10),
            Self::CompleteSignup { .. } => Duration::from_hours(1),
        }
    }

    /// Returns a duration that guarantees all tokens issued under this scope
    /// will be expired after the returned time passes.
    pub fn safe_scope_lifetime(&self) -> Duration {
        self.lifetime() + Duration::from_secs(1)
    }

    /// Returns a upper-bound duration after which any token (regardless of
    /// scope) is guaranteed to be expired.
    pub fn safe_global_lifetime() -> Duration {
        Duration::from_hours(24 * 7)
    }
}

impl DecodedToken {
    /// Returns the remaining time until this token is guaranteed to expire.
    pub fn safe_lifetime(&self) -> Duration {
        std::cmp::max(
            (self.expires_at - checked_now())
                .to_std()
                .unwrap_or(Duration::from_secs(0)),
            Duration::from_secs(1),
        )
    }
}

impl From<DecodedToken> for Token {
    fn from(value: DecodedToken) -> Self {
        Self {
            id: value.id,
            scope: value.scope,
        }
    }
}
