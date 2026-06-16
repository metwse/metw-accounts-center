use crate::{
    client::mock::{Mails, MockCaptchaClientImpl, MockMailClientImpl},
    dto,
    handlers::{AuthenticationHandler, HandlerResult},
    id::{AccountId, snowflake},
    repo::mock::{MockAccountRepoImpl, MockTokenRepoImpl},
    service::{AccountService, TokenService},
    state::State,
    util::mails,
};
use std::{cmp::max, sync::Arc};
use tokio::sync::Mutex;

/// Generate a random username string.
pub fn random_username() -> &'static str {
    let username = format!("{}", snowflake() as u64);

    format!(
        "user{}",
        &username[max(username.len() - 16, 0)..username.len()]
    )
    .leak()
}

/// Generate a random email string.
pub fn random_email() -> &'static str {
    format!("user{}@example.com", snowflake() as u64).leak()
}

/// Test repositories, handlers and clients.
#[allow(missing_docs)]
pub struct TestCtx {
    pub state: State,
    pub emails: Arc<Mutex<Mails>>,
}

impl Default for TestCtx {
    fn default() -> Self {
        let account_service = AccountService::new(MockAccountRepoImpl::boxed_new());
        let token_service =
            TokenService::new(MockTokenRepoImpl::boxed_new(), b"secret123".to_vec());
        let (emails, mail_client) = MockMailClientImpl::boxed_new();
        let capcha_client = MockCaptchaClientImpl::boxed_new();

        Self {
            state: State {
                account_service: account_service.into(),
                token_service: token_service.into(),
                mail_client: mail_client.into(),
                captcha_client: capcha_client.into(),
            },
            emails,
        }
    }
}

impl TestCtx {
    /// Creates a new test context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a random account.
    ///
    /// Returns `(id, username, email)`
    pub async fn signup(
        &self,
        client_password_hash: &'static str,
    ) -> (AccountId, &'static str, &'static str) {
        let username = random_username();
        let email = random_email();

        let account_id = AuthenticationHandler(self.state.clone())
            .signup(dto::request::Signup {
                client_password_hash: client_password_hash.to_string(),
                username: username.to_string(),
                email: email.to_string(),
                keys: dto::request::Keys {
                    identity_key: vec![1],
                    encrypted_private_key: vec![2],
                    encrypted_master_key: vec![3],
                },
            })
            .await
            .unwrap();

        (account_id, username, email)
    }

    /// Login with username.
    pub async fn login_with_username(
        &self,
        username: &'static str,
        client_password_hash: &'static str,
    ) -> HandlerResult<String> {
        AuthenticationHandler(self.state.clone())
            .login_by_username(dto::request::LoginWithUsername {
                username: username.to_string(),
                client_password_hash: client_password_hash.to_string(),
            })
            .await
    }

    /// Login with email.
    pub async fn login_with_email(
        &self,
        email: &'static str,
        client_password_hash: &'static str,
    ) -> HandlerResult<String> {
        AuthenticationHandler(self.state.clone())
            .login_by_email(dto::request::LoginWithEmail {
                email: email.to_string(),
                client_password_hash: client_password_hash.to_string(),
            })
            .await
    }

    /// Get the last email sent to the account.
    pub async fn last_email(&self, account_id: AccountId) -> mails::Template {
        let emails = self.emails.lock().await;
        let mailbox = emails.get(&account_id).unwrap();

        mailbox.last().unwrap().clone()
    }
}
