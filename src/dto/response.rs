/// Account entity returned to user after authentication.
pub struct Account {
    /// Unique user id.
    pub id: i64,

    /// User's primary username, if exists.
    pub username: Option<String>,
    /// Primary email address, if exitsts.
    pub email: Option<String>,

    /// Non-expiring username aliases.
    pub username_aliases: Vec<String>,
    /// Secondary emails.
    pub secondary_emails: Vec<String>,

    /// Account's criptographic keys.
    pub keys: Keys,
}

pub struct Keys {
    pub identity_key: Vec<u8>,
    pub encrypted_private_key: Vec<u8>,
    pub encrypted_master_key: Vec<u8>,
}
