use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::AccountId,
    state::AppState,
    token::{Token, TokenScope},
    util::emails,
};
use std::net::IpAddr;
use validator::Validate;

/// Gateway handlers for creating accouts or logging into accounts.
pub struct AuthenticationHandler(pub AppState);

impl AuthenticationHandler {
    /// Verify the session token.
    ///
    /// *This handler is intended for middleware.*
    #[tracing::instrument(skip_all)]
    pub async fn auth_session(self, base64_encoded_token: String) -> HandlerResult<AccountId> {
        let token = self.0.token_service.verify(&base64_encoded_token).await?;

        if let TokenScope::Session = token.scope {
            Ok(token.id)
        } else {
            Err(HandlerError::Unauthorized)
        }
    }

    /// Verify the pending activation session token.
    ///
    /// *This handler intended is for middleware.*
    #[tracing::instrument(skip_all)]
    pub async fn auth_email_verification_session(
        self,
        base64_encoded_token: String,
    ) -> HandlerResult<AccountId> {
        let token = self.0.token_service.verify(&base64_encoded_token).await?;

        if let TokenScope::EmailVerificationSession = token.scope {
            Ok(token.id)
        } else {
            Err(HandlerError::Unauthorized)
        }
    }

    /// Signs up a new account.
    ///
    /// Creates an unverified account and sends a [`ConfirmSignup`] email.
    ///
    /// [`ConfirmSignup`]: emails::Template::ConfirmSignup
    #[tracing::instrument(skip_all, fields(username = signup_dto.username, email = signup_dto.email))]
    pub async fn signup(
        self,
        signup_dto: dto::request::Signup,
        ip: IpAddr,
        captcha: dto::request::Captcha,
    ) -> HandlerResult<dto::response::Token> {
        signup_dto.validate()?;
        if !self.0.captcha_client.validate(captcha.captcha).await {
            return Err(HandlerError::InvalidCaptcha);
        }

        let email = signup_dto.email.clone();
        let username = signup_dto.username.clone();

        self.0
            .email_limiting_service
            .check_and_consume_quota(&ip, &email)
            .await?;

        let account_id = match self.0.account_service.signup(&signup_dto).await {
            Ok(account_id) => account_id,
            Err(err) => {
                self.0
                    .email_limiting_service
                    .refund_ip_quota(&ip, &email)
                    .await?;

                return Err(err)?;
            }
        };

        let complete_signup_jwt = self.0.token_service.sign(&Token {
            id: account_id,
            scope: TokenScope::CompleteSignup {
                email: email.clone(),
            },
        });

        let email_verification_session_jwt = self.0.token_service.sign(&Token {
            id: account_id,
            scope: TokenScope::EmailVerificationSession,
        });

        let template = emails::Template::ConfirmSignup {
            username,
            token: complete_signup_jwt,
        };

        self.0.email_client.send(email, account_id, template).await;

        Ok(dto::response::Token {
            token: email_verification_session_jwt,
        })
    }

    /// Returns a session JWT, with [`TokenScope::Session`] or
    /// [`TokenScope::EmailVerificationSession`] scope.
    #[tracing::instrument(skip_all, fields(username = login_dto.username))]
    pub async fn login_with_username(
        self,
        login_dto: dto::request::LoginWithUsername,
    ) -> HandlerResult<dto::response::Token> {
        login_dto.validate()?;

        let login = self
            .0
            .account_service
            .login_with_username(&login_dto)
            .await?;

        Ok(self.login(login))
    }

    /// Returns a session JWT, with [`TokenScope::Session`] or
    /// [`TokenScope::EmailVerificationSession`] scope.
    #[tracing::instrument(skip_all, fields(email = login_dto.email))]
    pub async fn login_with_email(
        self,
        login_dto: dto::request::LoginWithEmail,
    ) -> HandlerResult<dto::response::Token> {
        login_dto.validate()?;

        let login = self.0.account_service.login_with_email(&login_dto).await?;

        Ok(self.login(login))
    }

    fn login(self, login: dto::service::Login) -> dto::response::Token {
        tracing::trace!(%login.id);

        let token_scope = if login.is_email_verified {
            TokenScope::Session
        } else {
            TokenScope::EmailVerificationSession
        };

        dto::response::Token {
            token: self.0.token_service.sign(&Token {
                id: login.id,
                scope: token_scope,
            }),
        }
    }

    /// Revokes the JWT.
    ///
    /// Both the session tokens and authorization tokens can be revoked using
    /// this.
    pub async fn logout(self, token_dto: dto::request::Token) -> HandlerResult<()> {
        self.0
            .token_service
            .check_and_revoke_token(&self.0.token_service.decode(&token_dto.token).await?)
            .await?;

        Ok(())
    }
}
