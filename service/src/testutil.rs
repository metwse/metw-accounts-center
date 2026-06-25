use crate::id::snowflake;
use std::{cmp::max, net::IpAddr};

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

/// Generate a random IpAddr::V6.
pub fn random_ipv6() -> IpAddr {
    let mut octets = [0; 16];
    let snowflake = snowflake();

    for (i, val) in octets.iter_mut().enumerate().take(8) {
        *val = ((snowflake >> (i * 8)) & 255) as u8
    }

    IpAddr::from(octets)
}
