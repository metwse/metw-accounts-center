use crate::{
    client::{CaptchaClient, EmailClient},
    service::{AccountService, TokenService},
};
use std::sync::Arc;

/// Application-wide state.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
    pub token_service: Arc<TokenService>,
    pub email_client: Arc<dyn EmailClient>,
    pub captcha_client: Arc<dyn CaptchaClient>,
}
