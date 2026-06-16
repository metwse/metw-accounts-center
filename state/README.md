# metw-accounts-center state
Check out the
[sqlx-cli](https://github.com/transact-rs/sqlx/tree/main/sqlx-cli#create-and-run-migrations)
documentation for database migration.

## Running Tests
By default, tests require database connection or a secret to connect 3rd party
integration is ignored.

```sh
# This will run only the mock-repo tests
cargo test
```

Use `--include-ignored` to run all tests:

```sh
cargo test -- --include-ignored
```

Tests read environment variables for connection URLs/secrets.
| Variable | Description |
|--|--|
| `DATABASE_URL` | PostgreSQL connection URL. |
| `REDIS_URL` | Redis connection URL. |

Some tests require human interaction, and they do not run even if
`--include-ignored` flag given. You should manually run those tests, using
`cargo run --example <test-name>`.

| Test| Description | Required Variables |
|--|--|--|
| `amazon-sesv2` | Send a verification email for adding a new address. | `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`, `NOREPLY_EMAIL_ADDRESS` |
