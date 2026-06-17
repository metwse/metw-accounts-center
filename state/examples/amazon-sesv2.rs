//! This example sends a dummy email to given email.

use state::MailClientImpl;
use std::io::{self, Write};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sesv2::Client::new(&config);

    let noreply_email_address = std::env::var("NOREPLY_EMAIL_ADDRESS").unwrap();
    let mail_client =
        MailClientImpl::boxed_new(client, noreply_email_address, "http://example.com".into());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    print!("Please enter the address to send email: ");
    io::stdout().flush().ok();

    let mut dest = String::new();

    io::stdin().read_line(&mut dest).unwrap();

    dest = dest.trim().into();

    mail_client
        .send(
            dest.clone(),
            0.into(),
            service::util::mails::Template::ConfirmNewEmail {
                email: dest,
                token: "none".to_string(),
            },
        )
        .await;
}
