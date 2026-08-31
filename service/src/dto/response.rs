use crate::{
    entity,
    id::{AccountId, ApplicationId},
};
use serde::Serialize;
use utoipa::ToSchema;

/// Account entity returned to user after authentication.
#[derive(Debug, Serialize, ToSchema)]
pub struct Account {
    /// Unique account id.
    pub account_id: AccountId,

    /// User's primary username, if exists.
    pub username: Option<String>,
    /// Primary email address, if exitsts.
    pub email: Option<String>,

    /// Non-expiring username aliases.
    pub username_aliases: Vec<String>,
    /// Secondary emails.
    pub secondary_emails: Vec<String>,
}

/// Key derivation functions used in account.
#[derive(Debug, Serialize, ToSchema)]
pub struct AccountKdf {
    /// Key derivation applied to login password, client-side.
    pub client_password_kdf: entity::ClientPasswordKdf,
}

/// Base64-encoded, usually returned after sign up or log in.
#[derive(Debug, Serialize, ToSchema)]
pub struct Token {
    pub token: String,
}

/// Application name and ID.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationSummary {
    pub application_id: ApplicationId,
    pub name: String,
}

/// Newly created application.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedApplication {
    pub application_id: ApplicationId,
    pub name: String,
    pub client_secret: String,
}

/// The new secret after the client secret roll.
#[derive(Debug, Serialize, ToSchema)]
pub struct RotatedClientSecret {
    pub client_secret: String,
}

/// Application consent.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationConsent {
    pub application_id: ApplicationId,
    pub name: String,

    pub created_at_timestamp: String,
}

/// Authorization status of the application.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationConsentStatus {
    pub is_authorized: bool,
}

/// Response for a successful authorization code exchange.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorizationCodeExchangeResult {
    pub account_id: AccountId,
}

/// Authorization code.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorizationCode {
    pub authorization_code: String,
}
