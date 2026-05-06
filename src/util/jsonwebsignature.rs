use crate::{
    id::AccountId,
    token::{Token, TokenScope},
};
use biscuit::{JWT, jwa, jws};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

/// JSON web signature (JWS).
pub struct JsonWebSignature {
    secret: jws::Secret,
}

#[derive(Deserialize, Serialize)]
struct PrivateClaims {
    scope: TokenScope,
    id: AccountId,
}

impl JsonWebSignature {
    /// Creates a new JWS verifier/signer for [`Token`].
    pub fn new(secret: Vec<u8>) -> Self {
        Self {
            secret: jws::Secret::Bytes(secret),
        }
    }

    /// Sign and encode the token.
    pub fn encode(&self, token: &Token) -> String {
        let now = Utc::now();

        let payload = biscuit::ClaimsSet::<PrivateClaims> {
            registered: biscuit::RegisteredClaims {
                expiry: Some((now + token.valid_for).into()),
                not_before: Some(now.into()),
                issued_at: Some(now.into()),
                ..Default::default()
            },
            private: PrivateClaims {
                scope: token.scope.clone(),
                id: token.id,
            },
        };
        let header = jws::Header::<biscuit::Empty> {
            registered: jws::RegisteredHeader {
                algorithm: jwa::SignatureAlgorithm::HS256,
                ..Default::default()
            },
            ..Default::default()
        };

        let jwt = JWT::new_decoded(header, payload);

        jwt.into_encoded(&self.secret)
            .unwrap()
            .encoded()
            .unwrap()
            .to_string()
    }

    /// Decode the token by verifying it.
    pub fn decode(&self, base64_encoded_token: &str) -> Option<(Token, Vec<u8>)> {
        let token =
            biscuit::JWT::<PrivateClaims, biscuit::Empty>::new_encoded(base64_encoded_token);

        let signature = token.signature().ok()?;

        let token = token
            .into_decoded(&self.secret, jwa::SignatureAlgorithm::HS256)
            .ok()?;

        let now = Utc::now();

        token.validate(biscuit::ValidationOptions::default()).ok()?;

        let payload = token.payload().unwrap();
        let expiry = *payload.registered.expiry.unwrap();

        let valid_for = if expiry > now + Duration::seconds(60) {
            (expiry - now).to_std().unwrap()
        } else {
            std::time::Duration::from_secs(60)
        };

        Some((
            Token {
                id: payload.private.id,
                scope: payload.private.scope.clone(),
                valid_for,
            },
            signature,
        ))
    }
}
