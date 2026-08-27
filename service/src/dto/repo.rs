use crate::{
    entity,
    id::{AccountId, AppId},
};
use sqlx::prelude::FromRow;
use std::time::Duration;

#[derive(FromRow)]
pub struct OwnedLoginCredentials {
    pub account_id: AccountId,
    pub is_email_verified: bool,
    pub password_hash: Option<String>,
    pub server_password_hash_algorithm: entity::ServerPasswordHashAlgorithm,
}

#[derive(Debug)]
pub enum EmailLimitingResult {
    IpLimited(Duration),
    EmailLimited(Duration),
    Allowed,
}

#[derive(Debug)]
pub struct OwnedBasicAppInfo {
    pub app_id: AppId,
    pub name: String,
}
