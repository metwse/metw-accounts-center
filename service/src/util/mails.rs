/// Mail templates.
///
/// See [`TokenScope`].
///
/// [`TokenScope`]: `crate::token::TokenScope`
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum Template {
    /// See [`CompleteSignup`].
    ///
    /// [`CompleteSignup`]: `crate::token::TokenScope::CompleteSignup`
    ConfirmSignup { username: String, token: String },

    /// See [`AddEmail`].
    ///
    /// [`AddEmail`]: `crate::token::TokenScope::AddEmail`
    ConfirmNewEmail {
        username: String,
        email: String,
        token: String,
    },

    /// See [`ChangePrimaryEmail`].
    ///
    /// [`ChangePrimaryEmail`]: `crate::token::TokenScope::ChangePrimaryEmail`
    ConfirmPrimaryEmailChange {
        username: String,
        current_primary_email: String,
        new_primary_email: String,
        token: String,
    },
}

impl Template {
    /// Get subject of the template.
    pub fn subject(&self) -> String {
        match self {
            Self::ConfirmSignup { .. } => "Verify your metw.cc account".to_string(),
            Self::ConfirmNewEmail { .. } => "Add email to your metw.cc account".to_string(),
            Self::ConfirmPrimaryEmailChange { .. } => "Confirm primary mail change".to_string(),
        }
    }

    /// Get email body of the template.
    pub fn body(&self, callback_url: &str) -> String {
        match self {
            Self::ConfirmSignup { username, token } => format!(
                "Hello {username}! Please verify your account by clicking: {callback_url}?token={token}"
            ),
            Self::ConfirmNewEmail {
                username,
                email,
                token,
            } => format!(
                "To add <{email}> as a secondary email to your account @{username}, please click the link: {callback_url}?token={token}"
            ),
            Self::ConfirmPrimaryEmailChange {
                username,
                current_primary_email,
                new_primary_email,
                token,
            } => format!(
                "Hello {username}, please confirm your account's primary email change (from <{current_primary_email}> to <{new_primary_email}>) by clicking the link: {callback_url}?token={token}"
            ),
        }
    }
}
