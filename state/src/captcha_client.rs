use async_trait::async_trait;
use cf_turnstile::{SiteVerifyRequest, TurnstileClient};
use service::client::CaptchaClient;

/// CAPTCHA client validates Cloudflare's Turnstile.
pub struct CaptchaClientImpl {
    client: TurnstileClient,
}

impl CaptchaClientImpl {
    /// Creates a new CAPTCHA client.
    pub fn boxed_new(secret: String) -> Box<Self> {
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

#[cfg(test)]
#[tokio::test]
#[ignore]
async fn cloudflare_captcha() {
    const ALWAYS_PASS: &str = "1x0000000000000000000000000000000AA";
    const ALWAYS_FAIL: &str = "2x0000000000000000000000000000000AA";
    const ALWAYS_FAIL_ALREADY_SPENT: &str = "3x0000000000000000000000000000000AA";

    assert!(
        CaptchaClientImpl::boxed_new(ALWAYS_PASS.into())
            .validate("123".into())
            .await
    );
    assert!(
        !CaptchaClientImpl::boxed_new(ALWAYS_FAIL.into())
            .validate("123".into())
            .await
    );
    assert!(
        !CaptchaClientImpl::boxed_new(ALWAYS_FAIL_ALREADY_SPENT.into())
            .validate("123".into())
            .await
    );
}
