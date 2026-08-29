//! Take a look at the [System Design](service#system-design) section in the
//! service crate.
//!
//! This documentation discusses the communication between the `repo`, `service`,
//! and `handler` layers.
//!
//!
//! ## `repo`
//!
//! The [`repo`] layer is responsible for data management.
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
//! | [`get_primary_email`] | read | Gets primary email of the account. |
//! | [`compare_and_set_primary_email`] | compare-and-swap | If the provided primary email is still the current primary email, changes the email. |
//!
//! - For relational entity repositories that expose a `*RepoTransaction`,
//!  insert, delete, update, and upsert operations must be implemented through
//!  that transaction type.
//!
//! | Function Example | Type | Description |
//! |--|--|--|
//! | [`insert_username`] | insert | Inserts a new username. |
//! | [`insert_default_flags`] | insert | Inserts default user flags. |
//!
//! - `RepoError` represents internal persistence failures. It may propagate
//!   through internal layers for tracing and error handling, but its details
//!   must be redacted before constructing an external response.
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
//! [`compare_and_set_primary_email`]: service::repo::AccountRepo::compare_and_set_primary_email
//! [`get_primary_email`]: service::repo::AccountRepo::get_primary_email
//! [`insert_username`]: service::repo::AccountRepoTransaction::insert_username
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
//! unit-of-works. Generally, the handler layer executes one or more
//! service calls and either propagates the service results directly or
//! translates them into specific handler errors based on the business outcome.
//!
//! As an example, error conditions in [`AuthenticationHandler::auth_session`] are:
//!
//! | Condition | Error Type |
//! |--|--|
//! | Token scope is not [`TokenScope::Session`] | [`HandlerError::Unauthorized`] |
//! | JWT-related issue (triggered from TokenService) | [`HandlerError::Service`] |
//!
//! - There are two types of handlers: those for HTTP endpoints and those for
//!   middleware.
//!
//! - In handlers that are HTTP endpoints, at first request validation happens.
//!   Structured request bodies and query parameters are represented by
//!   [`dto::request`] types and validated by handlers. Path parameters and
//!   headers may use dedicated domain types or extractors. Successful responsei
//!   bodies are represented by [`dto::response`] types.
//!
//! - Middleware handlers may return types other than [`dto::response`].
//!
//! - Handlers own request DTOs and generally pass borrowed values to services.
//!   When an external client must retain request data, the handler transfers
//!   ownership to that client.
//!
//! [`handlers`]: service::handlers
//!
//! [`ServiceError`]: service::service::ServiceError
//! [`HandlerError::Unauthorized`]: service::handlers::HandlerError::Unauthorized
//! [`HandlerError::Service`]: service::handlers::HandlerError::Service
//!
//! [`TokenScope::Session`]: service::token::TokenScope::Session
//! [`dto::request`]: service::dto::request
//! [`dto::response`]: service::dto::response
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
