use crate::{
    client::{
        MailClient,
        impls::{MockMailClientImpl, mock::Mails},
    },
    dto,
    handlers::{AuthenticationHandler, AuthorizationHandler, HandlerResult, PersonalHandler},
    id::{AccountId, snowflake},
    repo::impls::{MockAccountRepoImpl, MockTokenRepoImpl},
    service::{AccountService, TokenService},
    util::templated_mails,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Generate a random username string.
pub fn random_username() -> &'static str {
    format!("user{}", snowflake() as u64).leak()
}

/// Generate a random email string.
pub fn random_email() -> &'static str {
    format!("user{}@example.com", snowflake() as u64).leak()
}

/// Test repositories, handlers and clients.
#[allow(missing_docs)]
pub struct TestCtx {
    pub account_service: Arc<AccountService>,
    pub token_service: Arc<TokenService>,
    pub mail_client: Arc<dyn MailClient>,
    pub emails: Arc<Mutex<Mails>>,
    pub authorization_handler: AuthorizationHandler,
    pub authentication_handler: AuthenticationHandler,
    pub personal_handler: PersonalHandler,
}

impl Default for TestCtx {
    fn default() -> Self {
        let account_service = Arc::new(AccountService::new(MockAccountRepoImpl::boxed_new()));
        let token_service = Arc::new(TokenService::new(
            MockTokenRepoImpl::boxed_new(),
            "supersecret123".into(),
        ));
        let (emails, mail_client) = MockMailClientImpl::shared_new_with_emails();

        let authorization_handler =
            AuthorizationHandler::new(Arc::clone(&account_service), Arc::clone(&token_service));

        let authentication_handler = AuthenticationHandler::new(
            Arc::clone(&account_service),
            Arc::clone(&token_service),
            Arc::clone(&mail_client),
            "http://example.com",
        );

        let personal_handler = PersonalHandler::new(
            Arc::clone(&account_service),
            Arc::clone(&token_service),
            Arc::clone(&mail_client),
            "http://example.com",
        );

        Self {
            account_service,
            token_service,
            mail_client,
            emails,
            authorization_handler,
            authentication_handler,
            personal_handler,
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
        password_hash: &'static str,
    ) -> (AccountId, &'static str, &'static str) {
        let username = random_username();
        let email = random_email();

        let account_id = self
            .authentication_handler
            .signup(dto::request::Signup {
                password_hash: password_hash.to_string(),
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
        password_hash: &'static str,
    ) -> HandlerResult<String> {
        self.authentication_handler
            .login_by_username(dto::request::LoginWithUsername {
                username: username.to_string(),
                password_hash: password_hash.to_string(),
            })
            .await
    }

    /// Login with email.
    pub async fn login_with_email(
        &self,
        email: &'static str,
        password_hash: &'static str,
    ) -> HandlerResult<String> {
        self.authentication_handler
            .login_by_email(dto::request::LoginWithEmail {
                email: email.to_string(),
                password_hash: password_hash.to_string(),
            })
            .await
    }

    /// Get the last email sent to the account.
    pub async fn last_email(&self, account_id: AccountId) -> templated_mails::Template {
        let emails = self.emails.lock().await;
        let mailbox = emails.get(&account_id).unwrap();

        mailbox.last().unwrap().clone()
    }
}
