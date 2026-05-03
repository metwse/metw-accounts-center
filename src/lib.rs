//! metw.cc accounts center

#![forbid(unsafe_code, unused_must_use)]
#![warn(clippy::all, missing_docs)]

/// Database entities.
pub mod entity;

/// Application storage.
pub mod repo;

/// Application state.
pub mod service;

/// Request handlers.
pub mod handlers;

/// Data transfer objects.
pub mod dto;

/// Miscellaneous utilites.
pub mod util;

/// Authentication: JWT & token scopes.
pub mod token;

/// External integration.
pub mod client;

/// Test utilities.
#[cfg(test)]
pub mod testutil;
