use crate::id::AccountId;
use sqlx::prelude::FromRow;

pub struct Keys<'a> {
    pub identity_key: &'a [u8],
    pub encrypted_private_key: &'a [u8],
    pub encrypted_master_key: &'a [u8],
}

pub struct OwnedKeys {
    pub identity_key: Vec<u8>,
    pub encrypted_private_key: Vec<u8>,
    pub encrypted_master_key: Vec<u8>,
}

#[derive(FromRow)]
pub struct OwnedLoginCredentials {
    pub id: AccountId,
    pub is_email_verified: bool,
    pub password_hash: String,
}

#[derive(Debug)]
pub enum EmailLimitingResult {
    IpTimeOut(usize),
    EmailTimeOut(usize),
    NoTimeOut,
}
