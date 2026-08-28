use crate::id::{AccountId, AppId};

/// Password verified login.
///
/// `is_email_verified` determines session type: [`Session`] if true,
/// [`EmailVerificationSession`] otherwise.
///
/// [`Session`]: crate::token::TokenScope::Session
/// [`EmailVerificationSession`]: crate::token::TokenScope::EmailVerificationSession
#[derive(Debug)]
pub struct Login {
    pub account_id: AccountId,
    pub is_email_verified: bool,
}

/// Newly created application.
pub struct NewApp {
    pub app_id: AppId,
    pub client_secret: String,
}
