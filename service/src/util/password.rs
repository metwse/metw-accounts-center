use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Argon2-hashed password.
pub async fn check(password: &str, hash: &str) -> bool {
    let password = password.to_string();
    let hash = hash.to_string();

    tokio::task::spawn_blocking(move || {
        let Ok(parsed_hash) = PasswordHash::new(&hash) else {
            return false;
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    })
    .await
    .unwrap()
}

/// Do Argon2 hasing on the password.
pub async fn hash(password: &str) -> String {
    let password = password.to_string();

    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    })
    .await
    .unwrap()
}

#[cfg(test)]
#[tokio::test]
#[test_log::test]
async fn test_hash() {
    let password = "very_very_strong_password";

    let hash = hash(password).await;

    assert!(check(password, &hash).await);

    assert!(!check(password, "tring_to_attack_with_inalid_hash_string").await);
}
