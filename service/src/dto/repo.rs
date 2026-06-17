use crate::id::AccountId;

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

pub struct OwnedLogin {
    pub id: AccountId,
    pub password_hash: String,
}
