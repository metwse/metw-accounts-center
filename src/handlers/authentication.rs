use crate::{
    client::MailClient,
    dto, entity,
    handlers::HandlerResult,
    service::{AccountService, TokenService},
    token::{Token, TokenScope},
    util::templated_mails,
};
use std::{sync::Arc, time::Duration};

static SIGNUP_TOKEN_VALID_FOR: Duration = Duration::from_secs(60 * 60 * 24);
static LOGIN_TOKEN_VALID_FOR: Duration = Duration::from_secs(60 * 60 * 24 * 7);

/// Gateway handlers for creating accouts or logging into accounts.
pub struct AuthenticationHandler {
    account_service: Arc<AccountService>,
    token_service: Arc<TokenService>,
    email_client: Arc<dyn MailClient>,
    signup_callback_url: &'static str,
}

impl AuthenticationHandler {
    /// Creates a new authentication hander.
    pub fn new(
        account_service: Arc<AccountService>,
        token_service: Arc<TokenService>,
        email_client: Arc<dyn MailClient>,
        signup_callback_url: &'static str,
    ) -> Self {
        Self {
            account_service,
            token_service,
            email_client,
            signup_callback_url,
        }
    }

    /// POST `/signup`
    pub async fn signup(
        &self,
        signup_dto: dto::request::Signup,
    ) -> HandlerResult<entity::AccountId> {
        let email = signup_dto.email.clone();
        let username = signup_dto.username.clone();

        let account_id = self.account_service.signup(signup_dto).await?;

        let signup_jwt = self.token_service.sign(&Token::new(
            account_id,
            vec![TokenScope::Signup { email }],
            SIGNUP_TOKEN_VALID_FOR,
        ));

        let template = templated_mails::Template::Signup {
            username,
            signup_jwt,
            callback_url: self.signup_callback_url,
        };

        self.email_client.send(account_id, template).await;

        Ok(account_id)
    }

    /// POST `/login` (with `username`)
    pub async fn login_by_username(
        &self,
        login_dto: dto::request::LoginWithUsername,
    ) -> HandlerResult<String> {
        let account_id = self.account_service.login_with_username(login_dto).await?;

        Ok(self.login(account_id))
    }

    /// POST `/login` (with `email`)
    pub async fn login_by_email(
        &self,
        login_dto: dto::request::LoginWithEmail,
    ) -> HandlerResult<String> {
        let account_id = self.account_service.login_with_email(login_dto).await?;

        Ok(self.login(account_id))
    }

    fn login(&self, account_id: entity::AccountId) -> String {
        self.token_service.sign(&Token::new(
            account_id,
            vec![TokenScope::Authenticate],
            LOGIN_TOKEN_VALID_FOR,
        ))
    }
}
