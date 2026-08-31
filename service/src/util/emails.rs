use crate::id::ApplicationId;
use serde::Serialize;

/// Email templates.
///
/// If the `email-templates` feature flag is enabled, the methods of this
/// struct return strings formatted for sending `multipart/HTML` emails. The
/// feature is not enabled, the templates are serialized as JSON. The
/// `email-templates` feature *should be enabled* in the production environment.
///
/// See also: [`TokenScope`].
///
/// [`TokenScope`]: `crate::token::TokenScope`
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize)]
pub enum Template {
    /// See [`CompleteSignup`].
    ///
    /// [`CompleteSignup`]: `crate::token::TokenScope::CompleteSignup`
    ConfirmSignup {
        username: String,
        token: String,
        application: Option<(ApplicationId, String)>,
    },

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

#[cfg(feature = "email-templates")]
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

#[cfg(feature = "email-templates")]
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
                Self::ConfirmSignup { username, token, application } => {
                    let callback_parameters = if let Some(application) = application {
                        let url_encoded_redirect_url = urlencoding::encode(&application.1);

                        format!(
                            "application_id={}&redirect_url={}&token={}",
                            application.0,
                            url_encoded_redirect_url,
                            token
                        )
                    } else {
                        format!("token={}", token)
                    };

                    format!(
                        get_template!($ty "confirm-signup"),
                        callback_url = callback_url,
                        username = username,
                        callback_parameters = callback_parameters,
                    )
                },
                Self::ConfirmNewEmail {
                    username,
                    token,
                    ..
                } => format!(
                    get_template!($ty "confirm-new-email"),
                    callback_url = callback_url,
                    username = username,
                    callback_parameters = format!("auth={}", token)
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
                    callback_parameters = format!("auth={}", token)
                ),
            }
        }
    };
}

impl Template {
    /// Get subject of the template.
    pub fn subject(&self) -> String {
        #[cfg(feature = "email-templates")]
        {
            build_email!(subject self).to_string()
        }

        #[cfg(not(feature = "email-templates"))]
        {
            serde_variant::to_variant_name(&self).unwrap().to_string()
        }
    }

    /// Get email body of the template.
    pub fn body_html(&self, callback_url: &str) -> String {
        #[cfg(feature = "email-templates")]
        {
            build_email!(body_html self, callback_url)
        }

        #[cfg(not(feature = "email-templates"))]
        {
            let json = serde_json::json!({
                "callback_url": callback_url,
                "template": self
            });

            serde_json::to_string(&json).unwrap()
        }
    }

    /// Get plaintext email body of the template.
    pub fn body_text(&self, callback_url: &str) -> String {
        #[cfg(feature = "email-templates")]
        {
            build_email!(body_text self, callback_url)
        }

        #[cfg(not(feature = "email-templates"))]
        {
            self.body_html(callback_url)
        }
    }
}
