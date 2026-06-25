use std::{collections::HashMap, sync::LazyLock};
use strfmt::strfmt;
use toml;

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

struct ProcessedTemplate {
    pub subject: String,
    pub body_text: String,
    pub body_html: String,
}

static TEMPLATES: LazyLock<HashMap<String, ProcessedTemplate>> = LazyLock::new(|| {
    let template_arguments = include_str!("../../email_templates/emails.toml");

    let html_template: &str = include_str!("../../email_templates/template.html");
    let text_template: &str = include_str!("../../email_templates/template.txt");

    let html_template = String::from_utf8(minify_html::minify(html_template.as_bytes(), &minify_html::Cfg::default())).unwrap();

    let templates =
        toml::from_str::<HashMap<String, HashMap<String, String>>>(template_arguments).unwrap();

    let mut processed_templates = HashMap::<String, ProcessedTemplate>::new();

    for (template_name, mut value) in templates.into_iter() {
        let body_html = strfmt(&html_template, &value).unwrap();
        let body_text = strfmt(text_template, &value).unwrap();

        processed_templates.insert(
            template_name,
            ProcessedTemplate {
                subject: value.remove("subject").unwrap(),
                body_text,
                body_html,
            },
        );
    }

    processed_templates
});

impl Template {
    /// Get subject of the template.
    pub fn subject(&self) -> String {
        TEMPLATES.get(self.to_str()).unwrap().subject.clone()
    }

    /// Get email body of the template.
    pub fn body_html(&self, callback_url: &str) -> String {
        let template = &TEMPLATES.get(self.to_str()).unwrap().body_html;

        self.build_email(template, callback_url)
    }

    /// Get plaintext email body of the template.
    pub fn body_text(&self, callback_url: &str) -> String {
        let template = &TEMPLATES.get(self.to_str()).unwrap().body_text;

        self.build_email(template, callback_url)
    }

    fn build_email(&self, template: &str, callback_url: &str) -> String {
        match self {
            Self::ConfirmSignup { username, token } => strfmt!(
                template,
                callback_url => callback_url.to_string(),
                token => token.to_string(),
                username => username.to_string()
            )
            .unwrap(),
            Self::ConfirmNewEmail {
                username,
                email,
                token,
            } => strfmt!(
                template,
                callback_url => callback_url.to_string(),
                token => token.to_string(),
                username => username.to_string(),
                email => email.to_string()
            )
            .unwrap(),
            Self::ConfirmPrimaryEmailChange {
                username,
                current_primary_email,
                new_primary_email,
                token,
            } => strfmt!(
                template,
                callback_url => callback_url.to_string(),
                token => token.to_string(),
                username => username.to_string(),
                current_primary_email => current_primary_email.to_string(),
                new_primary_email => new_primary_email.to_string()
            )
            .unwrap(),
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::ConfirmSignup { .. } => "confirm-signup",
            Self::ConfirmNewEmail { .. } => "confirm-new-email",
            Self::ConfirmPrimaryEmailChange { .. } => "confirm-primary-email-change",
        }
    }
}

#[test]
fn email_templates() {
    TEMPLATES.contains_key("");
}
