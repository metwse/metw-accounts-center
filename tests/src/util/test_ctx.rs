use service::{
    AppState,
    client::mock::{Mails, MockCaptchaClientImpl, MockMailClientImpl},
    dto,
    handlers::{AuthenticationHandler, AuthorizationHandler, HandlerResult},
    id::AccountId,
    repo::{
        AccountRepo, TokenRepo,
        mock::{MockAccountRepoImpl, MockTokenRepoImpl},
    },
    service::{AccountService, TokenService},
    testutil::{random_email, random_username},
    util::mails,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test repositories, handlers and clients.
#[allow(missing_docs)]
pub struct TestCtx {
    pub state: AppState,
    emails: Arc<Mutex<Mails>>,
}

impl Default for TestCtx {
    fn default() -> Self {
        let account_service = AccountService::new(MockAccountRepoImpl::boxed_new());
        let token_service =
            TokenService::new(MockTokenRepoImpl::boxed_new(), b"secret123".to_vec());
        let (emails, mail_client) = MockMailClientImpl::boxed_new();
        let capcha_client = MockCaptchaClientImpl::boxed_new();

        Self {
            state: AppState {
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

    /// Set the token repository.
    pub fn with_token_repo(mut self, token_repo: Box<dyn TokenRepo>) -> Self {
        self.state.token_service = TokenService::new(token_repo, b"secret123".to_vec()).into();

        self
    }

    /// Set the account repository.
    pub fn with_account_repo(mut self, account_repo: Box<dyn AccountRepo>) -> Self {
        self.state.account_service = AccountService::new(account_repo).into();

        self
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

    /// Creates a random account and verifies its email.
    pub async fn signup_and_verify_email(
        &self,
        client_password_hash: &'static str,
    ) -> (AccountId, &'static str, &'static str) {
        let (account_id, username, email) = self.signup(client_password_hash).await;

        let mails::Template::ConfirmSignup {
            token: complete_signup_jwt,
            ..
        } = self.last_email(account_id).await
        else {
            unreachable!()
        };

        AuthorizationHandler(self.state.clone())
            .auth(complete_signup_jwt)
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
            .login_with_username(dto::request::LoginWithUsername {
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
            .login_with_email(dto::request::LoginWithEmail {
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
