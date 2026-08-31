use rand::RngExt;

/// Password utilities.
pub mod password;

/// Email templates.
pub mod emails;

/// Authentication and privileged access tokens.
pub mod token;

/// Unique identifier types and the ID generation algorithm.
pub mod id;

/// Generate and validate client secrets.
pub mod client_secret;

mod jsonwebsignature;

pub use jsonwebsignature::JsonWebSignature;

/// Creates a new authorization code.
pub fn random_authorization_code() -> String {
    rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(22)
        .map(char::from)
        .collect()
}
