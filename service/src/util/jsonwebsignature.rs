#[cfg(test)]
use crate::checked_now;
#[cfg(not(test))]
use crate::checked_now;
use crate::{
    id::AccountId,
    token::{DecodedToken, Token, TokenScope},
};
use biscuit::{JWT, jwa, jws};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use chrono::TimeDelta;
#[cfg(test)]
use std::sync::Mutex;

/// JSON web signature (JWS).
pub struct JsonWebSignature {
    secret: jws::Secret,

    #[cfg(test)]
    time_delta: Mutex<TimeDelta>,
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

            #[cfg(test)]
            time_delta: Mutex::new(TimeDelta::zero()),
        }
    }

    /// Sign and encode the token.
    pub fn encode(&self, token: &Token) -> String {
        let now = self.now();

        let payload = biscuit::ClaimsSet::<PrivateClaims> {
            registered: biscuit::RegisteredClaims {
                expiry: Some((now + token.scope.lifetime()).into()),
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
    ///
    /// *Decoded token must never be encoded again!*
    pub fn decode(&self, base64_encoded_token: &str) -> Option<DecodedToken> {
        let token =
            biscuit::JWT::<PrivateClaims, biscuit::Empty>::new_encoded(base64_encoded_token);

        let signature = token.signature().ok()?;

        let token = token
            .into_decoded(&self.secret, jwa::SignatureAlgorithm::HS256)
            .ok()?;

        let now = self.now();

        token
            .validate(biscuit::ValidationOptions {
                claim_presence_options: biscuit::ClaimPresenceOptions {
                    issued_at: biscuit::Presence::Required,
                    not_before: biscuit::Presence::Required,
                    expiry: biscuit::Presence::Required,
                    ..Default::default()
                },
                temporal_options: biscuit::TemporalOptions {
                    now: Some(now),
                    ..Default::default()
                },
                ..Default::default()
            })
            .ok()?;

        let payload = token.payload().unwrap();
        let expires_at = *payload.registered.expiry.unwrap();
        let issued_at = *payload.registered.issued_at.unwrap();

        Some(DecodedToken {
            id: payload.private.id,
            scope: payload.private.scope.clone(),
            fingerprint: signature,
            expires_at,
            issued_at,
        })
    }

    #[cfg(test)]
    pub(crate) fn add_time_delta(&self, time_delta: TimeDelta) {
        *self.time_delta.lock().unwrap() += time_delta;
    }

    fn now(&self) -> DateTime<Utc> {
        #[cfg(test)]
        let now = checked_now() + *self.time_delta.lock().unwrap();

        #[cfg(not(test))]
        let now = checked_now();

        now
    }
}

#[cfg(test)]
#[test]
fn invalid_jwts() {
    let jws = JsonWebSignature::new("supersecret1234".into());

    assert!(jws.decode("aaa.aaa.aaa").is_none());
    assert!(jws.decode("aaa.aaa").is_none());
    assert!(jws.decode("a").is_none());
}
