use crate::{
    dto,
    handlers::{AuthenticationHandler, HandlerResult},
    id::{AccountId, snowflake},
    state::State,
    util::templated_mails,
};

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
    pub state: State,
}

impl Default for TestCtx {
    fn default() -> Self {
        let state = State::new_mock();

        Self { state }
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

        let account_id = AuthenticationHandler(self.state.clone())
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
        AuthenticationHandler(self.state.clone())
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
        AuthenticationHandler(self.state.clone())
            .login_by_email(dto::request::LoginWithEmail {
                email: email.to_string(),
                password_hash: password_hash.to_string(),
            })
            .await
    }

    /// Get the last email sent to the account.
    pub async fn last_email(&self, account_id: AccountId) -> templated_mails::Template {
        let emails = self.state.emails.lock().await;
        let mailbox = emails.get(&account_id).unwrap();

        mailbox.last().unwrap().clone()
    }
}
