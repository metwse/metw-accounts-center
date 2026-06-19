use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::AccountId,
    state::AppState,
    token::{Token, TokenScope},
    util::mails,
};
use validator::Validate;

/// Gateway handlers for creating accouts or logging into accounts.
pub struct AuthenticationHandler(pub AppState);

impl AuthenticationHandler {
    /// Verify the session token.
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
    #[tracing::instrument(skip_all)]
    pub async fn auth_pending_activation_session(
        self,
        base64_encoded_token: String,
    ) -> HandlerResult<AccountId> {
        let token = self.0.token_service.verify(&base64_encoded_token).await?;

        if let TokenScope::PendingActivationSession = token.scope {
            Ok(token.id)
        } else {
            Err(HandlerError::Unauthorized)
        }
    }

    /// Signs up a new account.
    ///
    /// Creates an unverified account and sends a [`ConfirmSignup`] email.
    ///
    /// [`ConfirmSignup`]: mails::Template::ConfirmSignup
    #[tracing::instrument(skip_all, fields(username = signup_dto.username, email = signup_dto.email))]
    pub async fn signup(self, signup_dto: dto::request::Signup) -> HandlerResult<AccountId> {
        signup_dto.validate()?;

        let email = signup_dto.email.clone();
        let username = signup_dto.username.clone();

        let account_id = self.0.account_service.signup(&signup_dto).await?;

        let complete_signup_jwt = self.0.token_service.sign(&Token::new(
            account_id,
            TokenScope::CompleteSignup {
                email: email.clone(),
            },
        ));

        let template = mails::Template::ConfirmSignup {
            username,
            token: complete_signup_jwt,
        };

        self.0.mail_client.send(email, account_id, template).await;

        Ok(account_id)
    }

    /// Returns a session JWT, with [`TokenScope::Session`] or
    /// [`TokenScope::PendingActivationSession`] scope.
    #[tracing::instrument(skip_all, fields(username = login_dto.username))]
    pub async fn login_by_username(
        self,
        login_dto: dto::request::LoginWithUsername,
    ) -> HandlerResult<String> {
        login_dto.validate()?;

        let login = self
            .0
            .account_service
            .login_with_username(&login_dto)
            .await?;

        Ok(self.login(login))
    }

    /// Returns a session JWT, with [`TokenScope::Session`] or
    /// [`TokenScope::PendingActivationSession`] scope.
    #[tracing::instrument(skip_all, fields(email = login_dto.email))]
    pub async fn login_by_email(
        self,
        login_dto: dto::request::LoginWithEmail,
    ) -> HandlerResult<String> {
        login_dto.validate()?;

        let login = self.0.account_service.login_with_email(&login_dto).await?;

        Ok(self.login(login))
    }

    fn login(self, login: dto::service::Login) -> String {
        tracing::trace!(%login.id);

        let token_scope = if login.is_verified {
            TokenScope::Session
        } else {
            TokenScope::PendingActivationSession
        };
        self.0
            .token_service
            .sign(&Token::new(login.id, token_scope))
    }

    /// Revokes the JWT.
    ///
    /// Both the session tokens and authorization tokens can be revoked using
    /// this.
    pub async fn logout(self, base64_encoded_token: String) -> HandlerResult<()> {
        self.0.token_service.revoke(&base64_encoded_token).await?;

        Ok(())
    }
}
