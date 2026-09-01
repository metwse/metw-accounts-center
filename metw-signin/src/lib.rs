#![doc = include_str!("../README.md")]

mod error;

pub use error::Error;

use reqwest::{Client as ReqwestClient, StatusCode};
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

static DEFAULT_BASE_URL: &str = "https://accounts.metw.cc/api";

/// A client for metw accounts center applications.
pub struct Client {
    client_secret: SecretBox<String>,
    application_id: String,
    http: ReqwestClient,
    base_url: Option<String>,
}

/// Represents a successful accounts response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique account ID.
    pub account_id: String,

    /// User's primary username, if exists.
    pub username: Option<String>,
    /// Primary email address, if exitsts.
    pub email: Option<String>,

    /// Username aliases.
    pub username_aliases: Option<Vec<String>>,
    /// Secondary emails.
    pub secondary_emails: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ExchangeResponse {
    account_id: String,
}

impl Client {
    /// Create a new metw client.
    pub fn new(application_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            application_id: application_id.into(),
            client_secret: SecretBox::new(Box::new(client_secret.into())),
            http: ReqwestClient::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(10))
                .user_agent(concat!("metw-signin/", env!("CARGO_PKG_VERSION"),))
                .build()
                .unwrap(),
            base_url: None,
        }
    }

    /// Set the API base URL.
    ///
    /// Optional, defaults to `https://accounts.metw.cc/api`.
    pub fn with_base_url(mut self, base_url: Option<String>) -> Self {
        self.base_url = base_url;

        self
    }

    fn get_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL)
    }

    /// Exchange authorization code with account ID.
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(
            application_id = %self.application_id,
            account_id = tracing::field::Empty
        ),
        err(level = "trace"),
    )]
    pub async fn exchange(&self, authorization_code: &str) -> Result<String, Error> {
        let res = self
            .http
            .post(format!("{}/application/exchange", self.get_base_url()))
            .basic_auth(
                &self.application_id,
                Some(self.client_secret.expose_secret()),
            )
            .json(&json!({ "authorization_code": authorization_code }))
            .send()
            .await?;

        let account_id = match res.status() {
            StatusCode::BAD_REQUEST => Err(Error::InvalidAuthorizationCode),
            StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
            StatusCode::OK => Ok(res.json::<ExchangeResponse>().await?.account_id),
            status => Err(Error::UnexpectedStatus(status)),
        }?;

        tracing::Span::current().record("account_id", &account_id);

        Ok(account_id)
    }

    /// Get account by its ID.
    #[tracing::instrument(
        level = "debug",
        skip(self),
        fields(application_id = %self.application_id),
        err(level = "trace"),
    )]
    pub async fn get_account(&self, account_id: &str) -> Result<Account, Error> {
        let res = self
            .http
            .get(format!(
                "{}/application/accounts/{account_id}",
                self.get_base_url()
            ))
            .basic_auth(
                &self.application_id,
                Some(self.client_secret.expose_secret()),
            )
            .send()
            .await?;

        match res.status() {
            StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
            StatusCode::OK => Ok(res.json().await?),
            status => Err(Error::UnexpectedStatus(status)),
        }
    }
}
