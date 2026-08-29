use crate::id::AccountId;
use serde::Deserialize;
use std::{str::FromStr, sync::LazyLock};
use utoipa::{IntoParams, PartialSchema, ToSchema};
use validator::Validate;

static USERNAME_REGEX_STR: &str = "^[a-z]([_.]?[a-z0-9])*$";
static REDIRECT_URL_REGEX_STR: &str = "^http([s]?)://";

static USERNAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(USERNAME_REGEX_STR).unwrap());

static REDIRECT_URL_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(REDIRECT_URL_REGEX_STR).unwrap());

fn validate_lowercase(s: &str) -> Result<(), validator::ValidationError> {
    if s.bytes().all(|b| !b.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("must_be_lowercase"))
    }
}

/// Sign up a new account.
///
/// Only PBKDF2-SHA256 is accepted.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct Signup {
    /// Username.
    #[validate(length(min = 2, max = 20), regex(path = *USERNAME_REGEX))]
    pub username: String,
    /// Email.
    #[validate(email, custom(function = validate_lowercase))]
    pub email: String,

    /// Password hash with KDF parameters.
    #[validate(nested)]
    pub password: ClientDerivedPassword,

    /// URL to redirect after a successful signup.
    #[validate(regex(path = *REDIRECT_URL_REGEX))]
    pub redirect_url: Option<String>,
}

/// Resend signup email.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct RetrySignup {
    /// Email.
    #[validate(email, custom(function = validate_lowercase))]
    pub email: String,

    /// URL to redirect after a successful signup.
    #[validate(regex(path = *REDIRECT_URL_REGEX))]
    pub redirect_url: Option<String>,
}

/// Password hashed client-side, with PBKDF2-SHA256 parameters.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
#[allow(missing_docs)]
pub struct ClientDerivedPassword {
    #[validate(length(max = 128))]
    pub base64_hash: String,

    #[validate(length(max = 128))]
    pub pbkdf2_salt: String,

    pub pbkdf2_iterations: u32,

    pub pbkdf2_length: u32,
}

/// Login into the account.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct Login {
    #[validate(nested)]
    pub account_identifier: AccountIdentifier,

    /// Argon2-hashed password.
    #[validate(length(max = 128))]
    pub client_password_hash: String,
}

/// Identify accounts by public or private identifiers.
#[derive(Debug, Clone)]
pub enum AccountIdentifier {
    Email(Email),
    Username(Username),
    AccountId(AccountId),
}

/// Identify accounts only by public identifiers.
#[derive(Debug, Clone)]
pub enum PublicAccountIdentifier {
    Username(Username),
    AccountId(AccountId),
}

/// Change the account password.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct ChangePassword {
    /// Current password.
    #[validate(length(max = 128))]
    pub current_password_hash: String,

    /// Password hash with KDF parameters.
    #[validate(nested)]
    pub new_password: ClientDerivedPassword,
}

/// Request only containing an username.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct Username {
    /// Username.
    #[validate(length(min = 2, max = 20), regex(path = *USERNAME_REGEX))]
    pub username: String,
}

/// Request only containing an email.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct Email {
    /// Email.
    #[validate(email, custom(function = validate_lowercase))]
    pub email: String,
}

/// Request containing JWT.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct Token {
    pub token: String,
}

/// Request containing CAPTCHA response.
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
pub struct Captcha {
    pub captcha: String,
}

/// Request for creating a new application or renaming one.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct ApplicationName {
    #[validate(length(min = 2, max = 32))]
    pub name: String,
}

/// Request containing redirect URL.
#[derive(Validate, Debug, Clone, Deserialize, ToSchema)]
pub struct ApplicationRedirectUrl {
    #[validate(url, regex(path = *REDIRECT_URL_REGEX))]
    pub redirect_url: String,
}

impl Validate for AccountIdentifier {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            Self::Email(email) => email.validate(),
            Self::Username(username) => username.validate(),
            Self::AccountId(_) => Ok(()),
        }
    }
}

impl PartialSchema for AccountIdentifier {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        String::schema()
    }
}

impl ToSchema for AccountIdentifier {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("String")
    }
}

impl AccountIdentifier {
    fn parse(s: String) -> Self {
        if let Ok(account_id) = AccountId::from_str(&s) {
            AccountIdentifier::AccountId(account_id)
        } else if s.contains('@') {
            AccountIdentifier::Email(Email { email: s })
        } else {
            AccountIdentifier::Username(Username { username: s })
        }
    }
}

impl<'de> Deserialize<'de> for AccountIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Ok(Self::parse(s))
    }
}

impl Validate for PublicAccountIdentifier {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            Self::Username(username) => username.validate(),
            Self::AccountId(_) => Ok(()),
        }
    }
}

impl PartialSchema for PublicAccountIdentifier {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        String::schema()
    }
}

impl ToSchema for PublicAccountIdentifier {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("String")
    }
}

impl PublicAccountIdentifier {
    fn parse(s: String) -> Self {
        if let Ok(account_id) = AccountId::from_str(&s) {
            PublicAccountIdentifier::AccountId(account_id)
        } else {
            PublicAccountIdentifier::Username(Username { username: s })
        }
    }
}

impl<'de> Deserialize<'de> for PublicAccountIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Ok(Self::parse(s))
    }
}

#[cfg(test)]
#[test]
fn username_regex() {
    let valids = ["pass", "p_a_s.s", "p_as.s123", "te_st"];

    let invalids = [
        "123test",
        "double__underscore",
        "_test",
        "end_",
        "end.",
        ".",
    ];

    for valid in valids {
        assert!((*USERNAME_REGEX).is_match(valid));
    }

    for invalid in invalids {
        assert!(!(*USERNAME_REGEX).is_match(invalid));
    }
}

#[cfg(test)]
#[test]
fn redirect_url_regex() {
    let valids = [
        "http://test",
        "https://123",
        "http://p/a/t/h",
        "http://////",
    ];

    let invalids = [
        "javascript:alert(1)",
        "htps://example.com",
        "localhost:8000/",
        "file:///home/metw",
    ];

    for valid in valids {
        assert!((*REDIRECT_URL_REGEX).is_match(valid));
    }

    for invalid in invalids {
        assert!(!(*REDIRECT_URL_REGEX).is_match(invalid));
    }
}

#[cfg(test)]
#[test]
fn email_validation() {
    let valids = ["test@example.com", "another_email@example.com"];

    let invalids = ["invalid.email", "NOUPPERCASE@example.com"];

    for valid in valids {
        assert!(
            Email {
                email: valid.into()
            }
            .validate()
            .is_ok()
        )
    }

    for invalid in invalids {
        assert!(
            Email {
                email: invalid.into()
            }
            .validate()
            .is_err()
        )
    }
}
