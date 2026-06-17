//! # metw-accounts-center service
//!
//! This crate is the abstract orchestrating microservice for metw.cc. It
//! manages clients and data repositories, and provides a library interface to
//! use with a presentation layer.
//!
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
//! categories above. Check out documentations of the `util` for details.
//! Conceptually, the [`token`] and [`id`] modules are re-exported at the crate
//! root as they are too common.
//!
//! [SoC]: https://en.wikipedia.org/wiki/Separation_of_concerns

/// Database entities.
///
/// One-to-one mapping of database entities to Rust types. Used internally --
/// between `repo` and `service` layers.
pub mod entity;

/// Data transfer objects.
///
/// Types for the inter-layer communication interface are defined by objects in
/// this module
pub mod dto;

/// Low-level definitions for application storage.
///
/// See [`_docs`] for the detailed documentation.
///
/// [`_docs`]: ../../_docs/acd_1/index.html#repo
pub mod repo;

/// An interface for manipulating a repo.
///
/// See [`_docs`] for the detailed documentation.
///
/// [`_docs`]: ../../_docs/acd_1/index.html#service
pub mod service;

/// An interface for manipulating the entire application state.
///
/// See [`_docs`] for the detailed documentation.
///
/// [`_docs`]: ../../_docs/acd_1/index.html#handlers
pub mod handlers;

/// Miscellaneous utilities.
pub mod util;

/// External integrations.
///
/// See [`_docs`] for the detailed documentation.
///
/// [`_docs`]: ../../_docs/acd_1/index.html#client
pub mod client;

mod state;

pub use util::{id, token};

pub use state::State;

/// Test utilities.
#[cfg(any(feature = "testutil", test))]
pub mod testutil;
