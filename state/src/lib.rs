//! # metw-accounts-center state
//!
//! This crate contains the state, *side effects*, of the [`service`] crate.
//! External integrations and data repository is served by this crate.
//!
//! ## Setup Recommendations
//!
//! - The token consumption and email rate limiting is enforced by Redis, in
//!   case of Redis state loss, one-time tokens can be accepted again. Make
//!   sure you have enabled persistent storage in Redis.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod captcha_client;

#[cfg(feature = "email-client-aws")]
mod aws_email_client;

#[cfg(feature = "email-client-lettre")]
mod lettre_email_client;

mod account_repo;
mod email_limiting_repo;
mod token_repo;

pub use captcha_client::CaptchaClientImpl;

#[cfg(feature = "email-client-aws")]
pub use aws_email_client::AwsEmailClientImpl;

#[cfg(feature = "email-client-lettre")]
pub use lettre_email_client::LettreEmailClientImpl;

pub use account_repo::AccountRepoImpl;
pub use email_limiting_repo::EmailLimitingRepoImpl;
pub use token_repo::TokenRepoImpl;

use serde::Deserialize;
use service::{
    AppState,
    client::{CaptchaClient, EmailClient},
    service::{AccountService, EmailLimitingService, TokenService},
};

/// Redis keys used with repositories.
#[cfg(any(feature = "testutil", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "testutil")))]
pub mod redis_keys {
    /// Keys used in token repository.
    pub mod token_repo {
        pub use crate::token_repo::{to_account_key, to_scope_key, to_token_key};
    }

    /// Keys used in email limiting repository.
    pub mod email_limiting_repo {
        pub use crate::email_limiting_repo::{
            to_block_email_key, to_block_ip_key, to_used_email_quota_key, to_used_ip_quota_key,
        };
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

    /// AWS SES key ID.
    #[cfg(feature = "email-client-aws")]
    pub aws_access_key_id: String,
    #[cfg(feature = "email-client-aws")]
    pub aws_secret_access_key: String,
    #[cfg(feature = "email-client-aws")]
    pub aws_region: String,

    /// SMTP server credentials.
    #[cfg(feature = "email-client-lettre")]
    pub smtp_relay: String,
    #[cfg(feature = "email-client-lettre")]
    pub smtp_username: String,
    #[cfg(feature = "email-client-lettre")]
    pub smtp_password: String,

    /// From address of emails sent by the email client.
    pub noreply_email_address: String,
    /// Callback URL for authorization tokens.
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

        let account_service = AccountService::new(AccountRepoImpl::boxed_new(pgpool));

        let redis = redis::Client::open(self.redis_url)
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        let token_service = TokenService::new(
            TokenRepoImpl::boxed_new(redis.clone()),
            self.jwt_secret.into(),
        );

        let email_limiting_service =
            EmailLimitingService::new(EmailLimitingRepoImpl::boxed_new(redis));

        #[cfg(all(feature = "email-client-aws", not(feature = "email-client-lettre")))]
        let email_client = {
            let aws_credentials = aws_credential_types::Credentials::new(
                self.aws_access_key_id,
                self.aws_secret_access_key,
                None,
                None,
                "MetwAccountsCenterConfig",
            );
            let aws_credentials_provider =
                aws_credential_types::provider::SharedCredentialsProvider::new(aws_credentials);

            let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .credentials_provider(aws_credentials_provider)
                .region(aws_config::Region::new(self.aws_region))
                .load()
                .await;

            let aws_sesv2_client = aws_sdk_sesv2::Client::new(&aws_config);

            AwsEmailClientImpl::boxed_new(
                aws_sesv2_client,
                self.noreply_email_address,
                self.email_callback_url,
            )
        };

        #[cfg(feature = "email-client-lettre")]
        let email_client = {
            use lettre::{
                AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials,
            };

            let creds = Credentials::new(self.smtp_username, self.smtp_password);

            let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_relay)
                .unwrap()
                .credentials(creds)
                .build();

            LettreEmailClientImpl::boxed_new(
                mailer,
                self.noreply_email_address,
                self.email_callback_url,
            )
        };

        let captcha_client = CaptchaClientImpl::boxed_new(self.cloudflare_turnstile_secret);

        AppState {
            account_service: account_service.into(),
            token_service: token_service.into(),
            email_limiting_service: email_limiting_service.into(),
            email_client: (email_client as Box<dyn EmailClient>).into(),
            captcha_client: (captcha_client as Box<dyn CaptchaClient>).into(),
        }
    }
}
