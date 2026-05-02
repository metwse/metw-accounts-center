//! metw.cc accounts center

#![forbid(unsafe_code, unused_must_use)]
#![warn(clippy::all, missing_docs)]

/// Database entities.
pub mod entity;

/// Application storage.
pub mod repo;

/// Application state.
pub mod service;

/// Data transfer objects.
pub mod dto;

mod snowflake;

pub use snowflake::{snowflake, EPOCH};
