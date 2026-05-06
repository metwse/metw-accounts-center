use crate::id::AccountId;

#[derive(Eq, PartialEq)]
pub struct Keys {
    pub identity_key: Vec<u8>,
    pub encrypted_private_key: Vec<u8>,
    pub encrypted_master_key: Vec<u8>,
}

pub struct Login {
    pub id: AccountId,
    pub password_hash: String,
}
