//! # metw-accounts-center state
//!
//! This crate contains the state, *side effects*, of the [`service`] crate.
//! External integrations and data repository is served by this crate.
//!
//! ## Setup Recommendations
//!
//! - The token consumption is enforced only by Redis revocation keys. In
//!   case of Redis state loss, one-time tokens can be accepted again.

mod captcha_client;
mod mail_client;

mod account_repo;
mod token_repo;

pub use captcha_client::CaptchaClientImpl;
pub use mail_client::MailClientImpl;

pub use account_repo::AccountRepoImpl;
pub use token_repo::TokenRepoImpl;

use serde::Deserialize;
use service::{
    AppState,
    service::{AccountService, TokenService},
};

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
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_region: String,

    /// From address of emails sent by the mail client.
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

        let token_service =
            TokenService::new(TokenRepoImpl::boxed_new(redis), self.jwt_secret.into());

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

        let mail_client = MailClientImpl::boxed_new(
            aws_sesv2_client,
            self.noreply_email_address,
            self.email_callback_url,
        );

        let captcha_client = CaptchaClientImpl::boxed_new(self.cloudflare_turnstile_secret);

        AppState {
            account_service: account_service.into(),
            token_service: token_service.into(),
            mail_client: mail_client.into(),
            captcha_client: captcha_client.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    // Test validity of the .env.example file
    #[test]
    #[serial_test::serial]
    fn config_from_example_env() {
        dotenvy::from_path_override("../.env.example").unwrap();

        Config::from_env();
    }

    // Bootstrap all services and clients, for testing .env
    #[tokio::test]
    #[ignore]
    #[serial_test::serial]
    async fn state_from_env() {
        dotenvy::dotenv_override().unwrap();

        let config = Config::from_env();

        config.bootstrap().await;
    }
}
