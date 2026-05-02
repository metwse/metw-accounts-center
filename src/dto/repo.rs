pub struct Keys {
    pub identity_key: Vec<u8>,
    pub encrypted_private_key: Vec<u8>,
    pub encrypted_master_key: Vec<u8>,
}

pub struct Login {
    pub id: i64,
    pub password_hash: String
}
