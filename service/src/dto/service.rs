use crate::id::AccountId;

/// Password verified login.
///
/// `is_verified` determines session type: [`Session`] if true,
/// [`PendingActivationSession`] otherwise.
///
/// [`Session`]: crate::token::TokenScope::Session
/// [`PendingActivationSession`]: crate::token::TokenScope::PendingActivationSession
#[derive(Debug)]
pub struct Login {
    pub id: AccountId,
    pub is_verified: bool,
}
