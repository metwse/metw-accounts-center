use crate::id::snowflake;
use std::cmp::max;

/// Generate a random username string.
pub fn random_username() -> &'static str {
    let username = format!("{}", snowflake() as u64);

    format!(
        "user{}",
        &username[max(username.len() - 16, 0)..username.len()]
    )
    .leak()
}

/// Generate a random email string.
pub fn random_email() -> &'static str {
    format!("user{}@example.com", snowflake() as u64).leak()
}
