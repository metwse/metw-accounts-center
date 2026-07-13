use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart},
};
use service::{client::EmailClient, id::AccountId, util::emails};
use tracing::{error, trace};

/// Email client for sending emails.
pub struct LettreEmailClientImpl {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_address: Mailbox,
    callback_url: String,
}

impl LettreEmailClientImpl {
    /// Creates a new Amazon SES v2 email client.
    pub fn boxed_new(
        mailer: AsyncSmtpTransport<Tokio1Executor>,
        from_address: String,
        callback_url: String,
    ) -> Box<Self> {
        Box::new(Self {
            mailer,
            from_address: Mailbox::new(None, from_address.parse().unwrap()),
            callback_url,
        })
    }
}

#[async_trait]
impl EmailClient for LettreEmailClientImpl {
    async fn send(&self, email: String, id: AccountId, template: emails::Template) {
        let _ = id;

        let Ok(dest_addr) = email.parse() else {
            error!(?email, "address passed to email client is not valid");

            return;
        };

        let dest = Mailbox::new(None, dest_addr);

        let Ok(msg) = Message::builder()
            .from(self.from_address.clone())
            .to(dest)
            .subject(template.subject())
            .multipart(MultiPart::alternative_plain_html(
                template.body_text(&self.callback_url),
                template.body_html(&self.callback_url),
            ))
        else {
            return;
        };

        let result = self.mailer.send(msg).await;

        if result.is_err() {
            trace!(?result, "could not send email")
        }
    }
}
