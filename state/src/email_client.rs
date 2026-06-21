use async_trait::async_trait;
use aws_sdk_sesv2 as sesv2;
use service::{client::EmailClient, id::AccountId, util::emails};
use tracing::{error, trace};

/// Email client for sending emails.
pub struct EmailClientImpl {
    client: sesv2::Client,
    from_address: String,
    callback_url: String,
}

impl EmailClientImpl {
    /// Creates a new Amazon SES v2 email client.
    pub fn boxed_new(
        client: sesv2::Client,
        from_address: String,
        callback_url: String,
    ) -> Box<Self> {
        Box::new(Self {
            client,
            from_address,
            callback_url,
        })
    }
}

#[async_trait]
impl EmailClient for EmailClientImpl {
    async fn send(&self, email: String, id: AccountId, template: emails::Template) {
        trace!(%id, ?template, "email to account");

        let dest = sesv2::types::Destination::builder()
            .to_addresses(email)
            .build();

        let Ok(subject_content) = sesv2::types::Content::builder()
            .data(template.subject())
            .charset("UTF-8")
            .build()
        else {
            return error!("cannot build subject");
        };

        let Ok(body_content) = sesv2::types::Content::builder()
            .data(template.body(&self.callback_url))
            .charset("UTF-8")
            .build()
        else {
            return error!("cannot build body");
        };

        let body = sesv2::types::Body::builder().text(body_content).build();

        let msg = sesv2::types::Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let email_content = sesv2::types::EmailContent::builder().simple(msg).build();

        let result = self
            .client
            .send_email()
            .from_email_address(&self.from_address)
            .destination(dest)
            .content(email_content)
            .send()
            .await;

        if result.is_err() {
            trace!(?result, "could not send email");
        }
    }
}
