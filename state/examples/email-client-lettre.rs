//! This example sends a dummy email to given address, using `lettre`.

use lettre::{AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials};
use service::client::EmailClient;
use state::LettreEmailClientImpl;
use std::io::{self, Write};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let noreply_email_address = std::env::var("NOREPLY_EMAIL_ADDRESS").unwrap();

    let relay = std::env::var("SMTP_RELAY").unwrap();
    let username = std::env::var("SMTP_USERNAME").unwrap();
    let password = std::env::var("SMTP_PASSWORD").unwrap();

    let creds = Credentials::new(username, password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&relay)
        .unwrap()
        .credentials(creds)
        .build();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let email_client = LettreEmailClientImpl::boxed_new(
        mailer,
        noreply_email_address,
        "http://example.com".to_owned(),
    );

    print!("Please enter the address to send email: ");
    io::stdout().flush().ok();

    let mut dest = String::new();

    io::stdin().read_line(&mut dest).unwrap();

    dest = dest.trim().into();

    email_client
        .send(
            dest.clone(),
            0.into(),
            service::util::emails::Template::ConfirmNewEmail {
                username: "metw".to_string(),
                email: dest,
                token: "none".to_string(),
            },
        )
        .await;
}
