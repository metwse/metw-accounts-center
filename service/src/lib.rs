//! # metw-accounts-center service
//!
//! This crate is the abstract orchestrating microservice for metw.cc. It
//! manages clients and data repositories, and  provides a library interface to
//! use with a presentation layer.
//!
//! Applications storing encrypted cryptographic secrets and serving users'
//! public keys to other services can use this service.
//!
//! This microservice must not be extended with non-authentication features.
//! Functionality of this microservice is limited only to username, email, and
//! password authentication.
//!
//! Goals:
//! - Authentication Service: A service can use this microservice as an
//!   authentication authority.
//! - End-to-End Encryption (E2EE): Users have a public key-private key pair
//!   and a master key. Their master keys and private keys are stored in the
//!   server, encrypted on the client side. The microservice publicly serves
//!   users' public keys; services that use this microservice for
//!   authentication can use those public keys to verify tokens signed by
//!   users.
//!
//! Non-Goals:
//! - Profiles, posts, shares, etc. The scope of this service is only
//!   authentication.
//!
//! ## System Design
//!
//! This project strictly adheres to the [Separation of Concerns][SoC] (SoC)
//! principle. The service abstraction levels are:
//! - [`repo`]: Defines traits for application storage, database, and cache
//!   access.
//! - [`service`]: Provides an interface for manipulating application state.
//!   Elaborates on low-level transactions defined in the `repo` layer to the
//!   upper layers.
//! - [`handlers`]: The final level of abstraction to manipulate the
//!   application state. This layer is a transport-agnostic front end for
//!   the microservice.
//! - [`State`]: The application state contains the services constructed from
//!   repositories and clients. Handlers require a state instance to construct.
//!
//! ### Supporting Modules
//!
//! [`client`] is used for external integrations (e.g., mail provider,
//! CAPTCHA).
//!
//! To ensure type-safety, the [`dto`] module defines the interface to
//! communicate with `handlers`. [`dto::repo`] and [`entity`] are used for
//! internal communication between `service` and `repo` layers.
//!
//! [`util`] is used for miscellaneous utilities that do not fit into
//! categories above. Check out documenatations of the `util` for details.
//! Conceptually, the [`token`] and [`id`] modules are re-exported at the crate
//! root as they are too common.
//!
//! [SoC]: https://en.wikipedia.org/wiki/Separation_of_concerns

#![forbid(unsafe_code, unused_must_use)]
#![warn(clippy::all, missing_docs)]

/// Database entities.
///
/// One-to-one mapping of database entities to Rust types. Used internally --
/// between `repo` and `service` layers.
pub mod entity;

/// Data transfer objects.
///
/// Types for inter-layer communication interface is defined by objects in this
/// module.
pub mod dto;

/// Low-level definitions for application storage.
///
/// This module defines the traits required to store persistent and volatile
/// data. Methods of `*Repo` traits provide read-only access or test-and-set
/// operations. `*Transaction` traits on the other hand, are atomic
/// transactional units to manipulate data freely.
pub mod repo;

/// Application state.
///
/// The service layer acts as the "middleman" between the handlers and the data
/// access, `repo`, layers. It abstracts low-level access operations into units
/// of works.
pub mod service;

/// Application state front end.
pub mod handlers;

/// Miscellaneous utilities.
pub mod util;

/// External integrations.
pub mod client;

mod state;

pub use util::{id, token};

pub use state::State;

/// Test utilities.
#[cfg(any(test, doc))]
pub mod testutil;
