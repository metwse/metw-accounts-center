use super::{HandlerError, HandlerResult};
use crate::{
    dto,
    id::AccountId,
    state::State,
    token::{Token, TokenScope},
    util::templated_mails,
};
use std::time::Duration;

static SIGNUP_TOKEN_VALID_FOR: Duration = Duration::from_secs(60 * 60 * 24);
static LOGIN_TOKEN_VALID_FOR: Duration = Duration::from_secs(60 * 60 * 24 * 7);

/// Gateway handlers for creating accouts or logging into accounts.
pub struct AuthenticationHandler(pub State);

impl AuthenticationHandler {
    /// Verify the authentication token.
    #[tracing::instrument(skip_all)]
    pub async fn auth(self, base64_encoded_token: String) -> HandlerResult<AccountId> {
        let token = self.0.token_service.verify(&base64_encoded_token).await?;

        if let TokenScope::Authenticate = token.scope {
            Ok(token.id)
        } else {
            Err(HandlerError::Unauthorized)
        }
    }

    /// POST `/signup`
    #[tracing::instrument(skip_all, fields(username = signup_dto.username, email = signup_dto.email))]
    pub async fn signup(self, signup_dto: dto::request::Signup) -> HandlerResult<AccountId> {
        let email = signup_dto.email.clone();
        let username = signup_dto.username.clone();

        let account_id = self.0.account_service.signup(signup_dto).await?;

        let signup_jwt = self.0.token_service.sign(&Token::new(
            account_id,
            TokenScope::Signup {
                email: email.clone(),
            },
            SIGNUP_TOKEN_VALID_FOR,
        ));

        let template = templated_mails::Template::Signup {
            username,
            signup_jwt,
            callback_url: self.0.email_callback_url,
        };

        self.0.mail_client.send(email, account_id, template).await;

        Ok(account_id)
    }

    /// POST `/login` (with `username`)
    #[tracing::instrument(skip_all, fields(username = login_dto.username))]
    pub async fn login_by_username(
        self,
        login_dto: dto::request::LoginWithUsername,
    ) -> HandlerResult<String> {
        let account_id = self
            .0
            .account_service
            .login_with_username(login_dto)
            .await?;

        Ok(self.login(account_id))
    }

    /// POST `/login` (with `email`)
    #[tracing::instrument(skip_all, fields(email = login_dto.email))]
    pub async fn login_by_email(
        self,
        login_dto: dto::request::LoginWithEmail,
    ) -> HandlerResult<String> {
        let account_id = self.0.account_service.login_with_email(login_dto).await?;

        Ok(self.login(account_id))
    }

    fn login(self, account_id: AccountId) -> String {
        tracing::trace!(%account_id);

        self.0.token_service.sign(&Token::new(
            account_id,
            TokenScope::Authenticate,
            LOGIN_TOKEN_VALID_FOR,
        ))
    }
}
