//! # metw-accounts-center state
//!
//! This crate implements the persistence and integration interfaces defined
//! by the `service` crate.
//!
//! ## Setup Recommendations
//!
//! - The token consumption and email rate limiting is enforced by Redis, in
//!   case of Redis state loss, one-time tokens can be accepted again. Make
//!   sure you have enabled persistent storage in Redis.
//!
//! ## Development Notes
//!
//! - Concurrent transactions *are not supported* on multiplexed Redis
//!   connections. Use a dedicated connection for each operation that requires
//!   a Redis transaction.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod captcha_client;
mod email_client;

mod account_repo;
mod application_repo;
mod authorization_code_repo;
mod email_limiting_repo;
mod token_repo;

pub use captcha_client::CaptchaClientImpl;
pub use email_client::EmailClientImpl;

pub use account_repo::AccountRepoImpl;
pub use application_repo::ApplicationRepoImpl;
pub use authorization_code_repo::AuthorizationCodeRepoImpl;
pub use email_limiting_repo::EmailLimitingRepoImpl;
pub use token_repo::TokenRepoImpl;

use serde::Deserialize;
use service::{
    AppState,
    client::{CaptchaClient, EmailClient},
    service::{
        AccountService, ApplicationService, AuthorizationCodeService, EmailLimitingService,
        TokenService,
    },
};

/// Redis keys used with repositories.
#[cfg(any(feature = "testutil", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "testutil")))]
pub mod redis_keys {
    /// Keys used in authorization code repository.
    pub mod authorization_code_repo {
        pub use crate::authorization_code_repo::to_authorization_code_key;
    }

    /// Keys used in email limiting repository.
    pub mod email_limiting_repo {
        pub use crate::email_limiting_repo::{
            to_block_email_key, to_block_ip_key, to_used_email_quota_key, to_used_ip_quota_key,
        };
    }

    /// Keys used in token repository.
    pub mod token_repo {
        pub use crate::token_repo::{to_account_key, to_scope_key, to_token_key};
    }
}

#[cfg(test)]
mod tests;

/// Config holds the configuration for the application.
#[derive(Clone, Debug, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    /// PostgreSQL connection URL
    pub database_url: String,
    /// Redis connection URL
    pub redis_url: String,

    /// Cloudflare Turnstile secret for CAPTCHA.
    pub cloudflare_turnstile_secret: String,

    /// SMTP server credentials.
    pub smtp_relay: String,
    pub smtp_username: String,
    pub smtp_password: String,

    /// From address of emails sent by the email client.
    pub noreply_email_address: String,
    /// Front end callback URL used in action emails.
    pub email_callback_url: String,

    /// JWT signature secret.
    pub jwt_secret: String,
}

impl Config {
    /// Reads the environment variables and returns a Config struct.
    pub fn from_env() -> Self {
        envy::from_env::<Self>().unwrap()
    }
}

impl Config {
    /// Initialize the [`service`] [`AppState`] from config.
    pub async fn bootstrap(self) -> AppState {
        let pgpool = sqlx::PgPool::connect(&self.database_url).await.unwrap();

        let account_service = AccountService::new(AccountRepoImpl::boxed_new(pgpool.clone()));
        let application_service = ApplicationService::new(ApplicationRepoImpl::boxed_new(pgpool));

        let redis_con_generator = Box::new(async move || {
            redis::Client::open(self.redis_url.clone())
                .unwrap()
                .get_multiplexed_async_connection()
                .await
                .unwrap()
        });

        let authorization_code_service = AuthorizationCodeService::new(
            AuthorizationCodeRepoImpl::boxed_new(&redis_con_generator).await,
        );

        let email_limiting_service =
            EmailLimitingService::new(EmailLimitingRepoImpl::boxed_new(&redis_con_generator).await);

        let token_service = TokenService::new(
            TokenRepoImpl::boxed_new(&redis_con_generator).await,
            self.jwt_secret.into(),
        );

        let email_client = {
            use lettre::{
                AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials,
            };

            let creds = Credentials::new(self.smtp_username, self.smtp_password);

            let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_relay)
                .unwrap()
                .credentials(creds)
                .build();

            EmailClientImpl::boxed_new(mailer, self.noreply_email_address, self.email_callback_url)
        };

        let captcha_client = CaptchaClientImpl::boxed_new(self.cloudflare_turnstile_secret);

        AppState {
            account_service: account_service.into(),
            application_service: application_service.into(),
            authorization_code_service: authorization_code_service.into(),
            token_service: token_service.into(),
            email_limiting_service: email_limiting_service.into(),
            email_client: (email_client as Box<dyn EmailClient>).into(),
            captcha_client: (captcha_client as Box<dyn CaptchaClient>).into(),
        }
    }
}
