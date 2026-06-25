use service::{
    AppState,
    client::{
        CaptchaClient, EmailClient,
        mock::{Emails, MockCaptchaClientImpl, MockEmailClientImpl},
    },
    dto,
    handlers::{AuthenticationHandler, AuthorizationHandler, HandlerResult},
    id::AccountId,
    repo::{
        AccountRepo, TokenRepo,
        mock::{MockAccountRepoImpl, MockEmailLimitingRepoImpl, MockTokenRepoImpl},
    },
    service::{AccountService, EmailLimitingService, TokenService},
    testutil::{random_email, random_ipv6, random_username},
    util::emails,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

/// Test repositories, handlers and clients.
#[allow(missing_docs)]
pub struct TestState {
    pub state: AppState,
    emails: Arc<Mutex<Emails>>,
}

impl Default for TestState {
    fn default() -> Self {
        let account_service = AccountService::new(MockAccountRepoImpl::boxed_new());
        let token_service =
            TokenService::new(MockTokenRepoImpl::boxed_new(), b"secret123".to_vec());
        let email_limiting_service =
            EmailLimitingService::new(MockEmailLimitingRepoImpl::boxed_new());
        let email_client = MockEmailClientImpl::boxed_new();
        let captcha_client = MockCaptchaClientImpl::boxed_new();

        let emails = email_client.get_emails();

        Self {
            state: AppState {
                account_service: account_service.into(),
                token_service: token_service.into(),
                email_limiting_service: email_limiting_service.into(),
                email_client: (email_client as Box<dyn EmailClient>).into(),
                captcha_client: (captcha_client as Box<dyn CaptchaClient>).into(),
            },
            emails,
        }
    }
}

impl TestState {
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

        let email_verification_jwt = AuthenticationHandler(self.state.clone())
            .signup(
                dto::request::Signup {
                    client_password_hash: client_password_hash.to_string(),
                    username: username.to_string(),
                    email: email.to_string(),
                    keys: dto::request::Keys {
                        identity_key: vec![1],
                        encrypted_private_key: vec![2],
                        encrypted_master_key: vec![3],
                    },
                },
                random_ipv6(),
            )
            .await
            .unwrap();

        let account_id = self
            .state
            .token_service
            .verify(&email_verification_jwt.token)
            .await
            .unwrap()
            .id;

        (account_id, username, email)
    }

    /// Creates a random account and verifies its email.
    pub async fn signup_and_verify_email(
        &self,
        client_password_hash: &'static str,
    ) -> (AccountId, &'static str, &'static str) {
        let (account_id, username, email) = self.signup(client_password_hash).await;

        let emails::Template::ConfirmSignup {
            token: complete_signup_jwt,
            ..
        } = self.last_email(account_id).await
        else {
            unreachable!()
        };

        AuthorizationHandler(self.state.clone())
            .auth(dto::request::Token {
                token: complete_signup_jwt,
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;

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
            .map(|jwt| jwt.token)
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
            .map(|jwt| jwt.token)
    }

    /// Get the last email sent to the account.
    pub async fn last_email(&self, account_id: AccountId) -> emails::Template {
        let emails = self.emails.lock().await;
        let mailbox = emails.get(&account_id).unwrap();

        mailbox.last().unwrap().clone()
    }
}
