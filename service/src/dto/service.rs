use crate::id::AccountId;

/// Password verified login.
///
/// `is_email_verified` determines session type: [`Session`] if true,
/// [`EmailVerificationSession`] otherwise.
///
/// [`Session`]: crate::token::TokenScope::Session
/// [`EmailVerificationSession`]: crate::token::TokenScope::EmailVerificationSession
#[derive(Debug)]
pub struct Login {
    pub id: AccountId,
    pub is_email_verified: bool,
}
