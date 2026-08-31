use crate::{
    client::{CaptchaClient, EmailClient},
    service::{
        AccountService, ApplicationService, AuthorizationCodeService, EmailLimitingService,
        TokenService,
    },
};
use std::sync::Arc;

/// Application-wide state.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct AppState {
    pub account_service: Arc<AccountService>,
    pub application_service: Arc<ApplicationService>,
    pub authorization_code_service: Arc<AuthorizationCodeService>,
    pub email_limiting_service: Arc<EmailLimitingService>,
    pub token_service: Arc<TokenService>,
    pub email_client: Arc<dyn EmailClient>,
    pub captcha_client: Arc<dyn CaptchaClient>,
}
