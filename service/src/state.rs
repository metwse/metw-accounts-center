use crate::{
    client::{CaptchaClient, MailClient},
    service::{AccountService, TokenService},
};
use std::sync::Arc;

/// Application-wide state.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
    pub token_service: Arc<TokenService>,
    pub mail_client: Arc<dyn MailClient>,
    pub captcha_client: Arc<dyn CaptchaClient>,
}
