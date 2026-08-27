use rand::RngExt;
use sha2::Digest;

/// Client secret.
#[allow(missing_docs)]
pub struct ClientSecret {
    pub client_secret: String,
    pub client_secret_hash: [u8; 32],
}

/// Creates a new client secret.
pub fn random_client_secret() -> String {
    rand::rng()
        .sample_iter(rand::distr::Alphabetic)
        .take(22)
        .map(char::from)
        .collect()
}

/// Hashes the client secret.
pub fn hash_client_secret(client_secret: &str) -> [u8; 32] {
    let mut client_secret_hash: [u8; 32] = [0; 32];

    let mut hasher = sha2::Sha256::new();
    hasher.update(client_secret);
    client_secret_hash.copy_from_slice(&hasher.finalize());

    client_secret_hash
}

/// Validates the client secret's hash.
pub fn validate_client_secret(client_secret: &str, client_secret_hash: &[u8; 32]) -> bool {
    let calculated_client_secret_hash = hash_client_secret(client_secret);

    let mut difference: u8 = 0;

    for i in 0..32 {
        difference |= calculated_client_secret_hash[i] ^ client_secret_hash[i];
    }

    difference == 0
}

#[cfg(test)]
#[test]
fn test() {
    for _ in 0..256 {
        let mut client_secret = random_client_secret();
        let mut client_secret_hash = hash_client_secret(&client_secret);

        assert!(validate_client_secret(&client_secret, &client_secret_hash));

        client_secret += "x";

        assert!(!validate_client_secret(&client_secret, &client_secret_hash));

        client_secret.pop();

        assert!(validate_client_secret(&client_secret, &client_secret_hash));

        client_secret_hash[0] ^= 255;

        assert!(!validate_client_secret(&client_secret, &client_secret_hash));
    }
}
