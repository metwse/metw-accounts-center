/// Email templates.
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

macro_rules! get_template {
    (subject $name:literal) => {
        get_template!(@ "-subject.txt" $name)
    };

    (body_html $name:literal) => {
        get_template!(@ ".html" $name)
    };

    (body_text $name:literal) => {
        get_template!(@ ".txt" $name)
    };

    (@ $exten:literal $name:expr) => {
        include_str!(concat!(env!("OUT_DIR"), "/email-templates_", $name, $exten))
    };
}

macro_rules! build_email {
    (subject $template:expr) => {
        match $template {
            Self::ConfirmSignup { .. } => get_template!(subject "confirm-signup"),
            Self::ConfirmNewEmail { .. } => get_template!(subject "confirm-new-email"),
            Self::ConfirmPrimaryEmailChange { .. } =>
                get_template!(subject "confirm-primary-email-change"),
        }
    };
    ($ty:tt $template:expr, $callback_url:expr) => {
        {
            let callback_url = $callback_url;

            match $template {
                Self::ConfirmSignup { username, token } => format!(
                    get_template!($ty "confirm-signup"),
                    callback_url = callback_url,
                    username = username,
                    token = token
                ),
                Self::ConfirmNewEmail {
                    username,
                    token,
                    ..
                } => format!(
                    get_template!($ty "confirm-new-email"),
                    callback_url = callback_url,
                    username = username,
                    token = token
                ),
                Self::ConfirmPrimaryEmailChange {
                    username,
                    current_primary_email,
                    new_primary_email,
                    token,
                } => format!(
                    get_template!($ty "confirm-primary-email-change"),
                    callback_url = callback_url,
                    username = username,
                    current_primary_email = current_primary_email,
                    new_primary_email = new_primary_email,
                    token = token
                ),
            }
        }
    };
}

impl Template {
    /// Get subject of the template.
    pub fn subject(&self) -> String {
        build_email!(subject self).to_string()
    }

    /// Get email body of the template.
    pub fn body_html(&self, callback_url: &str) -> String {
        build_email!(body_html self, callback_url)
    }

    /// Get plaintext email body of the template.
    pub fn body_text(&self, callback_url: &str) -> String {
        build_email!(body_text self, callback_url)
    }
}
