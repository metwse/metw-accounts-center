use crate::{
    client::{CaptchaClient, EmailClient},
    service::{AccountService, AppService, EmailLimitingService, TokenService},
};
use std::sync::Arc;

/// Application-wide state.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
    pub app_service: Arc<AppService>,
    pub token_service: Arc<TokenService>,
    pub email_limiting_service: Arc<EmailLimitingService>,
    pub email_client: Arc<dyn EmailClient>,
    pub captcha_client: Arc<dyn CaptchaClient>,
}
