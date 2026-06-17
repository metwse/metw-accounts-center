use crate::{
    repo::TokenRepo,
    service::{ServiceError, ServiceResult},
    token::Token,
    util::JsonWebSignature,
};

/// Token state.
pub struct TokenService {
    repo: Box<dyn TokenRepo>,
    jws: JsonWebSignature,
}

impl TokenService {
    /// Creates a new token service.
    pub fn new(repo: Box<dyn TokenRepo>, secret: Vec<u8>) -> Self {
        Self {
            repo,
            jws: JsonWebSignature::new(secret),
        }
    }

    /// Sign the token.
    pub fn sign(&self, token: &Token) -> String {
        self.jws.encode(token)
    }

    /// Validate and decode the token.
    #[tracing::instrument(skip_all)]
    pub async fn verify(&self, base64_encoded_token: &str) -> ServiceResult<Token> {
        if let Some((token, signature)) = self.jws.decode(base64_encoded_token) {
            if !self.repo.check_revocation(&signature).await? {
                Ok(token)
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
        if let Some((token, signature)) = self.jws.decode(base64_encoded_token) {
            if !self
                .repo
                .check_and_revoke(&signature, token.lifetime)
                .await?
            {
                Ok(token)
            } else {
                Err(ServiceError::TokenRevoked)
            }
        } else {
            Err(ServiceError::InvalidJwt)
        }
    }
}
