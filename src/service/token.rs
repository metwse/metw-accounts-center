use crate::{
    repo::TokenRepo,
    service::{ServiceError, ServiceResult},
    token::Token,
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, jws::Jws};
use std::{marker::PhantomData, time::Duration};

/// Token state.
pub struct TokenService {
    pub(super) repo: Box<dyn TokenRepo>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl TokenService {
    /// Creates a new token service.
    pub fn new(repo: Box<dyn TokenRepo>, secret: &[u8]) -> Self {
        let encoding_key = EncodingKey::from_secret(secret);
        let decoding_key = DecodingKey::from_secret(secret);

        Self {
            repo,
            encoding_key,
            decoding_key,
        }
    }

    /// Sign the token.
    pub fn sign(&self, token: &Token) -> String {
        jsonwebtoken::encode(&Header::default(), &token, &self.encoding_key).unwrap_or("".into())
    }

    /// Validate and decode the token.
    #[tracing::instrument(skip_all)]
    pub async fn verify(&self, base64_encoded_token: &str) -> ServiceResult<Token> {
        let mut it = base64_encoded_token.split(".");
        let protected = it.next().ok_or(ServiceError::InvalidJwt).unwrap();
        let payload = it.next().ok_or(ServiceError::InvalidJwt)?;
        let signature = it.next().ok_or(ServiceError::InvalidJwt)?;

        let jws: Jws<Token> = Jws {
            protected: protected.into(),
            payload: payload.into(),
            signature: signature.into(),
            _pd: PhantomData,
        };

        // hope the compiler optimize
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.validate_nbf = true;

        if let Ok(token) = jsonwebtoken::jws::decode(&jws, &self.decoding_key, &validation) {
            if !self.repo.check_revocation(signature.as_bytes()).await? {
                Ok(token.claims)
            } else {
                Err(ServiceError::TokenRevoked)
            }
        } else {
            Err(ServiceError::InvalidJwt)
        }
    }

    /// Revoke the token
    #[tracing::instrument(skip_all)]
    pub async fn revoke(&self, base64_encoded_token: &str) -> ServiceResult<Token> {
        let token = self.verify(base64_encoded_token).await?;

        let now = Utc::now().timestamp() as u64;
        let exp = token.exp as u64;

        // Revoke already expired tokens if they are already expired.
        let duration = if now > exp - 60 { 60 } else { exp - now };

        let signature = base64_encoded_token.split(".").nth(2).unwrap();

        self.repo
            .revoke(signature.as_bytes(), Duration::from_secs(duration))
            .await?;

        Ok(token)
    }
}
