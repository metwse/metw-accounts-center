#[cfg(test)]
use tokio::sync::Mutex;

use crate::{
    client::MailClient,
    service::{AccountService, TokenService},
};
#[cfg(test)]
use crate::{
    client::impls::{MockMailClientImpl, mock::Mails},
    repo::impls::{MockAccountRepoImpl, MockTokenRepoImpl},
};
use std::{env, sync::Arc};

/// Config holds the configuration for the application.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Config {
    pub email_callback_url: String,
    pub jwt_secret: String,

    pub database_url: String,

    pub host: String,
    pub port: u16,
}

impl Config {
    /// Reads the environment variables and returns a Config struct.
    #[tracing::instrument]
    pub fn from_env() -> Self {
        let email_callback_url = env::var("EMAIL_CALLBACK_URL")
            .expect("EMAIL_CALLBACK_URL environment variable is not found");

        let jwt_secret =
            env::var("JWT_SECRET").expect("JWT_SECRET environment variable is not found");

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not found");

        let port: u16 = {
            if let Ok(port) = env::var("PORT") {
                tracing::info!("PORT not given, defaulting to 4003");

                port.parse().expect("invalid port")
            } else {
                4003
            }
        };

        let host = {
            if let Ok(host) = env::var("HOST") {
                tracing::info!("HOST not given, defaulting to localhost");

                host
            } else {
                "localhost".to_string()
            }
        };

        Self {
            email_callback_url,
            jwt_secret,
            database_url,
            host,
            port,
        }
    }
}

/// Application-wide state.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct State {
    pub account_service: Arc<AccountService>,
    pub token_service: Arc<TokenService>,
    pub mail_client: Arc<dyn MailClient>,
    pub email_callback_url: Arc<String>,
    #[cfg(test)]
    pub emails: Arc<Mutex<Mails>>,
}

impl State {
    /// Initializes application state from given configuration.
    pub fn new(_config: Config) -> Self {
        todo!()
    }

    /// Creates a new mock state.
    #[cfg(any(test, doc))]
    pub fn new_mock() -> Self {
        let account_service = AccountService::new(MockAccountRepoImpl::boxed_new());
        let token_service =
            TokenService::new(MockTokenRepoImpl::boxed_new(), b"secret123".to_vec());
        let (emails, mail_client) = MockMailClientImpl::boxed_new();

        Self {
            account_service: account_service.into(),
            token_service: token_service.into(),
            mail_client: mail_client.into(),
            email_callback_url: Arc::new("http://example.com".to_string()),
            emails,
        }
    }
}
