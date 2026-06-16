# metw-accounts-center state
Production repository and client implementations for the `service` crate.

## Setup
Required environment variables for the state are listed in fields of
[Config](https://metwse.github.io/metw-accounts-center/state/struct.Config.html)
struct.

Run database migrations:
```sh
cd state/

cargo sqlx migrate run
```
(check out
[sqlx-cli](https://github.com/transact-rs/sqlx/tree/main/sqlx-cli#create-and-run-migrations)
documentation for sqlx installation)

> **Important:** To ensure application security, carefully read the
  [Setup Recommendations](https://metwse.github.io/metw-accounts-center/state/index.html#setup-recommendations)
  section.

## Tests
`cargo test` runs mock-only tests with no external dependencies. To include
tests that require live services (require `.env`):
```sh
cargo test -- --include-ignored
```

Some tests require human interaction and are run as examples:
```sh
cargo run --example <name>
```

| Example | Description |
|--|--|
| `amazon-sesv2` | Send a verification email for adding a new address. |
