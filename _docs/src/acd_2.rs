//!
//! The points you should keep in mind when writing a test are:
//!
//! - Persistent storage can be used in repository tests. Use randomized values
//!   for fields that may have unique constraints.
//!
//! - Use `#[ignore]` for tests that require a live production database or an
//!   external client, and write tests within examples for those that require
//!   human interaction.
//!
//! - Write tests that require repository access in the [`tests`] module, see
//!   its documentation for details.
//!
//!
//! [`tests`]: tests
