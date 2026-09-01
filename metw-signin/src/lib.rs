#![doc = include_str!("../README.md")]

mod error;

pub use error::Error;

use reqwest::{Client as ReqwestClient, StatusCode};
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

/// A client for metw accounts center applications.
pub struct Client {
    client_secret: SecretBox<String>,
    application_id: String,
    http: ReqwestClient,
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
    pub username_aliases: Vec<String>,
    /// Secondary emails.
    pub secondary_emails: Vec<String>,
}

#[derive(Deserialize)]
struct ExchangeResponse {
    account_id: String,
}

impl Client {
    /// Create a new metw client.
    pub fn new<T: fmt::Display, U: fmt::Display>(application_id: T, client_secret: U) -> Self {
        Self {
            application_id: format!("{application_id}"),
            client_secret: SecretBox::new(Box::new(format!("{client_secret}"))),
            http: ReqwestClient::builder()
                .user_agent(concat!(
                    "metw-signin (",
                    env!("CARGO_PKG_HOMEPAGE"),
                    ", ",
                    env!("CARGO_PKG_VERSION"),
                    ")",
                ))
                .build()
                .unwrap(),
        }
    }

    /// Exchange authorization code with account ID.
    pub async fn exchange<T: Serialize>(&self, authorization_code: T) -> Result<String, Error> {
        let res = self
            .http
            .post("https://accounts.metw.cc/api/application/exchange")
            .basic_auth(
                &self.application_id,
                Some(self.client_secret.expose_secret()),
            )
            .json(&json!({ "authorization_code": authorization_code }))
            .send()
            .await?;

        match res.status() {
            StatusCode::BAD_REQUEST => return Err(Error::InvalidAuthorizationCode),
            StatusCode::UNAUTHORIZED => return Err(Error::Unauthorized),
            status if status != StatusCode::OK => return Err(Error::UnknownError),
            _ => (),
        };

        let res: ExchangeResponse = res.json().await?;

        Ok(res.account_id)
    }

    /// Get account by its ID.
    pub async fn get_account<T: fmt::Display>(&self, account_id: T) -> Result<Account, Error> {
        let res = self
            .http
            .get(format!(
                "https://accounts.metw.cc/api/application/accounts/{account_id}"
            ))
            .basic_auth(
                &self.application_id,
                Some(self.client_secret.expose_secret()),
            )
            .send()
            .await?;

        if res.status() != StatusCode::OK {
            return Err(Error::Unauthorized);
        }

        Ok(res.json().await?)
    }
}
