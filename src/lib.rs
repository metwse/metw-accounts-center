//! metw.cc accounts center

#![forbid(unsafe_code, unused_must_use)]
#![warn(clippy::all, missing_docs)]

/// Database entities.
pub mod entity;

/// Persistent storage.
pub mod repo;

/// Data transfer objects.
pub mod dto;
