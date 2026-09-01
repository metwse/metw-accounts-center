# metw-signin
A Rust client for metw accounts center applications.

## Example
```rust
use metw_signin::{Client, Account};

let my_application_id = "12356";
let my_client_secret = "abcdef";

let client = Client::new(my_application_id, my_client_secret);

// Your callback endpoint, i.e.
// Query(AuthorizationParams { authorization_code }): Query<AuthorizationParams>
let account_id = client.exchange(authorization_code).await?;
let account = client.get_account(account_id).await?;
```
