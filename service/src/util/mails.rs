/// Mail templates.
///
/// See [`TokenScope`].
///
/// [`TokenScope`]: `crate::token::TokenScope`
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum Template {
    ConfirmSignup {
        username: String,
        token: String,
    },

    ConfirmNewEmail {
        email: String,
        token: String,
    },

    ConfirmPrimaryEmailChange {
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
            Self::ConfirmNewEmail { email, token } => format!(
                "To add <{email}> as a secondary email to your account, please click the link: {callback_url}?token={token}"
            ),
            Self::ConfirmPrimaryEmailChange {
                current_primary_email,
                new_primary_email,
                token,
            } => format!(
                "Please confirm your account's primary email change (from <{current_primary_email}> to <{new_primary_email}>) by clicking the link: {callback_url}?token={token}"
            ),
        }
    }
}
