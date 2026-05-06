use std::sync::LazyLock;
use validator::Validate;

static USERNAME_REGEX_STR: &str = "^[a-z]([_.]?[a-z0-9])*$";

static USERNAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(USERNAME_REGEX_STR).unwrap());

/// Sign up a new account.
#[derive(Validate, Debug, Clone)]
pub struct Signup {
    /// Username.
    #[validate(length(min = 2, max = 20), regex(path = *USERNAME_REGEX))]
    pub username: String,
    /// Email.
    #[validate(email)]
    pub email: String,
    /// Argon2-hashed password.
    #[validate(length(max = 128))]
    pub password_hash: String,

    /// Initial keys assigned when account is created.
    #[validate(nested)]
    pub keys: Keys,
}

/// Login into the account.
#[derive(Validate, Debug, Clone)]
pub struct LoginWithUsername {
    /// Username.
    #[validate(length(min = 2, max = 20), regex(path = *USERNAME_REGEX))]
    pub username: String,

    /// Argon2-hashed password.
    #[validate(length(max = 128))]
    pub password_hash: String,
}

/// Login into the account.
#[derive(Validate, Debug, Clone)]
pub struct LoginWithEmail {
    /// Email.
    #[validate(email)]
    pub email: String,

    /// Argon2-hashed password.
    #[validate(length(max = 128))]
    pub password_hash: String,
}

/// Roll keys, change password, master key or key pair.
#[derive(Validate, Debug, Clone)]
pub struct KeyRoll {
    /// Argon2-hashed password. Password will not be changed if its empty.
    #[validate(length(max = 128))]
    pub password_hash: String,

    /// New keys.
    #[validate(nested)]
    pub keys: Keys,
}

/// Account's criptographic keys.
#[derive(Validate, Debug, Clone)]
pub struct Keys {
    /// Curve25519 public key in der format.
    #[validate(length(max = 2048))]
    pub identity_key: Vec<u8>,
    /// Private pair of identity key, ecrypted by master key.
    #[validate(length(max = 2048))]
    pub encrypted_private_key: Vec<u8>,
    /// Master key, encrypted by password.
    #[validate(length(max = 2048))]
    pub encrypted_master_key: Vec<u8>,
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
