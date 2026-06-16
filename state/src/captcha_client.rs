use async_trait::async_trait;
use cf_turnstile::{SiteVerifyRequest, TurnstileClient};
use service::client::CaptchaClient;

/// CAPTCHA client validates Cloudflare's Turnstile.
pub struct CaptchaClientImpl {
    client: TurnstileClient,
}

impl CaptchaClientImpl {
    /// Creates a new CAPTCHA client.
    pub fn boxed_new(secret: String) -> Box<dyn CaptchaClient> {
        Box::new(Self {
            client: TurnstileClient::new(secret.into()),
        })
    }
}

#[async_trait]
impl CaptchaClient for CaptchaClientImpl {
    async fn validate(&self, id: String) -> bool {
        let Ok(validated) = self
            .client
            .siteverify(SiteVerifyRequest {
                response: id,
                ..Default::default()
            })
            .await
        else {
            return false;
        };

        validated.success
    }
}
