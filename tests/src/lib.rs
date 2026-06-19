//! # metw-accounts-center tests
//!
//! This crate contains test scenarios that run with either production or mock
//! repository and client implementations. By default, `cargo test` uses only
//! mock implementations. To include tests that require live services (requires
//! a `.env` file):
//! ```sh
//! cargo test -- --include-ignored
//! ```
//!
//! Some tests require human interaction and run as examples:
//! ```sh
//! # Sends a verification email for adding a new address.
//! cargo run --example amazon-sesv2
//! ```
//!
//! See [`ACD-2`](_docs::acd_2) for test guideline.
