//! # metw-accounts-center state
//!
//! This crate contains the state, *side effects*, of the [`service`] crate.
//! External integrations and data repository is served by this crate.

use service::State;
use std::env;

mod captcha_client;
mod mail_client;

mod account_repo;
mod token_repo;

#[cfg(test)]
mod tests;

pub use captcha_client::CaptchaClientImpl;
pub use mail_client::MailClientImpl;

pub use account_repo::AccountRepoImpl;
pub use token_repo::TokenRepoImpl;

/// Config holds the configuration for the application.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Config {
    pub email_callback_url: String,
    pub jwt_secret: String,
    pub cloudflare_turnstile_secret: String,

    pub database_url: String,
}

impl Config {
    /// Reads the environment variables and returns a Config struct.
    #[tracing::instrument]
    pub fn from_env() -> Self {
        let email_callback_url = env::var("EMAIL_CALLBACK_URL")
            .expect("EMAIL_CALLBACK_URL environment variable is not found");

        let jwt_secret =
            env::var("JWT_SECRET").expect("JWT_SECRET environment variable is not found");

        let cloudflare_turnstile_secret = env::var("CLOUDFLARE_TURNSTILE_SECRET")
            .expect("CLOUDFLARE_TURNSTILE_SECRET environment variable is not found");

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not found");

        Self {
            email_callback_url,
            jwt_secret,
            cloudflare_turnstile_secret,
            database_url,
        }
    }

    /// Initialize the [`service`] [`State`] from config.
    pub async fn bootstrap(self) -> State {
        todo!()
    }
}
