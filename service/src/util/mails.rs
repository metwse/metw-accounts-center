use std::sync::Arc;

/// Mail templates.
///
/// See [`TokenScope`].
///
/// [`TokenScope`]: `crate::token::TokenScope`
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum Template {
    Signup {
        username: String,
        signup_jwt: String,
        callback_url: Arc<String>,
    },

    AddEmail {
        email: String,
        add_email_jwt: String,
        callback_url: Arc<String>,
    },

    SetPrimaryEmail {
        current_primary_email: String,
        new_primary_email: String,
        set_primary_mail_jwt: String,
        callback_url: Arc<String>,
    },
}

impl Template {
    /// Get subject of the template.
    pub fn subject(&self) -> String {
        match self {
            Self::Signup { .. } => "Verify your metw.cc account".to_string(),
            Self::AddEmail { .. } => "Add email to your metw.cc account".to_string(),
            Self::SetPrimaryEmail { .. } => "Confirm primary mail change".to_string(),
        }
    }

    /// Get email body of the template.
    pub fn body(&self) -> String {
        match self {
            Self::Signup {
                username,
                signup_jwt,
                callback_url,
            } => format!(
                "Hello {username}! Please verify your account by clicking: {callback_url}?token={signup_jwt}"
            ),
            Self::AddEmail {
                email,
                add_email_jwt,
                callback_url,
            } => format!(
                "To add <{email}> as a secondary email to your account, please click the link: {callback_url}?token={add_email_jwt}"
            ),
            Self::SetPrimaryEmail {
                current_primary_email,
                new_primary_email,
                set_primary_mail_jwt,
                callback_url,
            } => format!(
                "Please confirm your account's primary email change (from <{current_primary_email}> to <{new_primary_email}>) by clicking the link: {callback_url}?token={set_primary_mail_jwt}"
            ),
        }
    }
}
