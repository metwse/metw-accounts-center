
use crate::{
    id::AccountId,
    token::{Token, TokenScope},
};
use biscuit::{JWT, jwa, jws};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use chrono::TimeDelta;
#[cfg(test)]
use std::sync::Mutex;

/// JSON web signature (JWS).
pub struct JsonWebSignature {
    secret: jws::Secret,

    #[cfg(test)]
    time_delta: Mutex<TimeDelta>
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
        let now = Utc::now();

        let payload = biscuit::ClaimsSet::<PrivateClaims> {
            registered: biscuit::RegisteredClaims {
                expiry: Some((now + token.lifetime).into()),
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
    /// *Decoded token must never encoded again!* The decoded token will have
    /// at least 60 seconds lifetime.
    pub fn decode(&self, base64_encoded_token: &str) -> Option<(Token, Vec<u8>)> {
        let token =
            biscuit::JWT::<PrivateClaims, biscuit::Empty>::new_encoded(base64_encoded_token);

        let signature = token.signature().ok()?;

        let token = token
            .into_decoded(&self.secret, jwa::SignatureAlgorithm::HS256)
            .ok()?;

        #[cfg(test)]
        let now = Utc::now() + *self.time_delta.lock().unwrap();

        #[cfg(not(test))]
        let now = Utc::now();

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
        let expiry = *payload.registered.expiry.unwrap();

        let lifetime = if expiry > now + Duration::seconds(60) {
            (expiry - now).to_std().unwrap()
        } else {
            std::time::Duration::from_secs(60)
        };

        Some((
            Token::new_with_lifetime(payload.private.id, payload.private.scope.clone(), lifetime),
            signature,
        ))
    }

    #[cfg(test)]
    pub(crate) fn add_time_delta(&self, time_delta: TimeDelta) {
        *self.time_delta.lock().unwrap() += time_delta;
    }
}
