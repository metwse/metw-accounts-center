//! Basic authorization example.

use metw_signin::{Client, Error};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut application_id = String::new();
    let mut client_secret = String::new();

    println!("Enter your application ID");
    io::stdin().read_line(&mut application_id).unwrap();

    println!("Enter your client secret");
    io::stdin().read_line(&mut client_secret).unwrap();

    let client = Client::new(application_id.trim(), client_secret.trim());

    let mut authorization_code = String::new();

    println!("Enter authorization code to fetch account: ");
    io::stdin().read_line(&mut authorization_code).unwrap();

    let account_id = client.exchange(authorization_code.trim()).await?;
    let account = client.get_account(account_id).await?;

    println!("{account:#?}");

    Ok(())
}
