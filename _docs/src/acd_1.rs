//! Take a look at the [System Design](service#system-design) section in the
//! service crate.
//!
//! This documentation discusses the communication between the `repo`, `service`,
//! and `handler` layers.
//!
//!
//! ## `repo`
//!
//! The [`repo`] layer is responsible for low-level state management.
//!
//! Methods of `*Repo` traits provide read-only access or test-and-set
//! operations. `*RepoTransaction` traits, on the other hand, are atomic
//! transactional units to manipulate data freely. For example, [`AccountRepo`]
//! or its transaction variant [`AccountRepoTransaction`]. Implementation
//! conventions for this layer:
//!
//! - Non-transaction repo operations can *only be* read or atomic
//!   compare-and-swap / test-and-set.
//!
//! | Function Example | Type | Description |
//! |--|--|--|
//! | [`get_account_flags`] | read | Reads user flags. |
//! | [`set_primary_email_if_current_is`] | compare-and-swap | If the provided primary email is still the current primary email, changes the email. |
//!
//! - Any insert, delete, update, or upsert (insert or update if exists)
//!   operation must reside within a transaction.
//!
//! | Function Example | Type | Description |
//! |--|--|--|
//! | [`upsert_account`] | upsert | Inserts a new user or updates the existing one by account id. |
//! | [`insert_default_flags`] | insert | Inserts default user flags. |
//!
//! - `*Repo` variants that return a single item like "fetch one" do not return
//!   a `RowNotFound` error; they must be mapped to an `Option`. Those that
//!   are "fetch many" return a `Vec`.
//!
//! | Function Example | Type | Description |
//! |--|--|--|
//! | [`get_primary_email`] | fetch one | Returns the primary email of the user. |
//! | [`get_secondary_emails`] | fetch many | Returns the secondary emails of the user. |
//!
//! - [`RepoError`] is an internal error used only for tracing and
//!   observability. This error must not be returned directly to the user; it
//!   must be *redacted*.
//!
//! [`repo`]: service::repo
//!
//! [`AccountRepo`]: service::repo::AccountRepo
//! [`AccountRepoTransaction`]: service::repo::AccountRepoTransaction
//!
//! [`RepoError`]: service::repo::RepoError
//!
//! [`set_primary_email_if_current_is`]: service::repo::AccountRepo::set_primary_email_if_current_is
//! [`get_account_flags`]: service::repo::AccountRepo::get_account_flags
//! [`upsert_account`]: service::repo::AccountRepoTransaction::upsert_account
//! [`insert_default_flags`]: service::repo::AccountRepoTransaction::insert_default_flags
//! [`get_primary_email`]: service::repo::AccountRepo::get_primary_email
//! [`get_secondary_emails`]: service::repo::AccountRepo::get_secondary_emails
//!
//!
//! ## `service`
//!
//! The [`service`] layer acts as the "middleman" between the handlers and the
//! data access (repo) layers. It abstracts low-level access operations into
//! units of work.
//!
//! - High-level business logic validation errors are mapped to specific
//!   variants of [`ServiceError`]. Low-level database infrastructure failures
//!   bypass specific mappings and propagate as generic errors.
//!
//! As an example, error conditions in [`AccountService::signup`] are:
//! | Condition | Error Type |
//! |--|--|
//! | Username is already taken | [`ServiceError::UsernameTaken`] |
//! | Email is already taken | [`ServiceError::EmailTaken`] |
//!
//! The service layer handles specific error mapping on a *best-effort* basis
//! by validating conditions before executing raw changes. If a validation
//! fails to predict a state conflict, or if an underlying database constraint
//! triggers an unhandled error, the fallback result will be a generic
//! [`ServiceError::Repo`]. In such scenarios, upper layers must continue to
//! treat the internal transparent errors as redacted.
//!
//! [`service`]: service::service
//!
//! [`AccountService::signup`]: service::service::AccountService::signup
//!
//! [`ServiceError`]: service::service::ServiceError
//! [`ServiceError::UsernameTaken`]: service::service::ServiceError::UsernameTaken
//! [`ServiceError::EmailTaken`]: service::service::ServiceError::EmailTaken
//! [`ServiceError::Repo`]: service::service::ServiceError::Repo
//!
//!
//! ## `handlers`
//!
//! The [`handlers`] layer composes multiple services to orchestrate complex,
//! high-level operations. Generally, the handler layer executes one or more
//! service calls and either propagates the service results directly or translates
//! them into specific handler errors based on the business outcome.
//!
//! As an example, error conditions in [`AuthenticationHandler::auth_session`] are:
//!
//! | Condition | Error Type |
//! |--|--|
//! | Token scope is not [`TokenScope::Session`] | [`HandlerError::Unauthorized`] |
//! | JWT-related issue (triggered from TokenService) | [`HandlerError::Service`] |
//!
//! - Request validation happens at the handlers. Everything coming from the
//!   user that requires validation *must be* a [`dto::request`] type or a JWT
//!   string.
//!
//! - The handler owns the allocation and passes references to the service
//!   layer. It, however, transfer ownership of the allocation to `client`s.
//!
//! [`handlers`]: service::handlers
//!
//! [`ServiceError`]: service::service::ServiceError
//! [`HandlerError::Unauthorized`]: service::handlers::HandlerError::Unauthorized
//! [`HandlerError::Service`]: service::handlers::HandlerError::Service
//!
//! [`TokenScope::Session`]: service::token::TokenScope::Session
//! [`dto::request`]: service::dto::request
//!
//! [`AuthenticationHandler::auth_session`]: service::handlers::AuthenticationHandler::auth_session
//!
//!
//! ## `client`
//!
//! [`client`] contains external integrations such as [`CaptchaClient`]. They
//! typically send requests to external services.
//!
//! [`client`]: service::client
//!
//! [`CaptchaClient`]: service::client::CaptchaClient
