use crate::{
    repo::TokenRepo,
    service::{ServiceError, ServiceResult},
    token::{DecodedToken, Token},
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
        let decoded_token = self.decode(base64_encoded_token).await?;

        if !self.repo.is_revoked(&decoded_token).await? {
            Ok(decoded_token.into())
        } else {
            Err(ServiceError::TokenRevoked)
        }
    }

    /// See [`TokenRepo::check_and_revoke_token`]. Maps revoked tokens to
    /// `Err(ServiceError::TokenRevoked)`
    #[tracing::instrument(skip_all)]
    pub async fn check_and_revoke_token(&self, decoded_token: &DecodedToken) -> ServiceResult<()> {
        if self.repo.check_and_revoke_token(decoded_token).await? {
            return Err(ServiceError::TokenRevoked);
        }

        Ok(())
    }

    /// See [`TokenRepo::check_and_revoke_account_tokens_with_scope`]. Maps
    /// revoked tokens to `Err(ServiceError::TokenRevoked)`
    #[tracing::instrument(skip_all)]
    pub async fn check_and_revoke_account_tokens_with_scope(
        &self,
        decoded_token: &DecodedToken,
    ) -> ServiceResult<()> {
        if self
            .repo
            .check_and_revoke_account_tokens_with_scope(decoded_token)
            .await?
        {
            return Err(ServiceError::TokenRevoked);
        }

        Ok(())
    }

    /// See [`TokenRepo::check_and_revoke_account_tokens`]. Maps revoked tokens
    /// to `Err(ServiceError::TokenRevoked)`
    #[tracing::instrument(skip_all)]
    pub async fn check_and_revoke_account_tokens(
        &self,
        decoded_token: &DecodedToken,
    ) -> ServiceResult<()> {
        if self
            .repo
            .check_and_revoke_account_tokens(decoded_token)
            .await?
        {
            return Err(ServiceError::TokenRevoked);
        }

        Ok(())
    }

    /// Decode the base64 encoded token and validate its signature and
    /// expiration.
    #[tracing::instrument(skip_all)]
    pub async fn decode(&self, base64_encoded_token: &str) -> ServiceResult<DecodedToken> {
        if let Some(decoded_token) = self.jws.decode(base64_encoded_token) {
            Ok(decoded_token)
        } else {
            Err(ServiceError::InvalidJwt)
        }
    }
}
