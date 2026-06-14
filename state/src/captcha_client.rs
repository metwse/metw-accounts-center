use async_trait::async_trait;
use service::client::CaptchaClient;

/// CAPTCHA client validates Cloudflare's Turnstile.
pub struct CaptchaClientImpl;

#[async_trait]
impl CaptchaClient for CaptchaClientImpl {
    async fn validate(&self, _id: String) -> bool {
        todo!()
    }
}
