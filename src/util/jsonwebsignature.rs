use crate::{
    id::AccountId,
    token::{Token, TokenScope},
};
use biscuit::{JWT, jwa, jws};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use std::sync::Mutex;

/// JSON web signature (JWS).
pub struct JsonWebSignature {
    secret: jws::Secret,
}

#[derive(Deserialize, Serialize)]
struct PrivateClaims {
    scope: TokenScope,
    id: AccountId,
}

#[cfg(not(test))]
static NOW: fn() -> DateTime<Utc> = Utc::now;
#[cfg(test)]
static NOW: fn() -> DateTime<Utc> = JsonWebSignature::injected_now;

#[cfg(test)]
static INJECTED_NOW: Mutex<Option<DateTime<Utc>>> = Mutex::new(None);

impl JsonWebSignature {
    /// Creates a new JWS verifier/signer for [`Token`].
    pub fn new(secret: Vec<u8>) -> Self {
        Self {
            secret: jws::Secret::Bytes(secret),
        }
    }

    /// Sign and encode the token.
    pub fn encode(&self, token: &Token) -> String {
        let now = NOW();

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

        let now = NOW();

        token
            .validate(biscuit::ValidationOptions {
                claim_presence_options: biscuit::ClaimPresenceOptions {
                    issued_at: biscuit::Presence::Required,
                    not_before: biscuit::Presence::Required,
                    expiry: biscuit::Presence::Required,
                    ..Default::default()
                },
                temporal_options: biscuit::TemporalOptions {
                    now: Some(NOW()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .ok()?;

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

    #[cfg(test)]
    pub(crate) fn inject_now(date_time: Option<DateTime<Utc>>) {
        *INJECTED_NOW.lock().unwrap() = date_time
    }

    #[cfg(test)]
    fn injected_now() -> DateTime<Utc> {
        if let Some(now) = *INJECTED_NOW.lock().unwrap() {
            now
        } else {
            Utc::now()
        }
    }
}
