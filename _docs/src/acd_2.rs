//!
//! The points you should keep in mind when writing a test are:
//!
//! - Tests that reading environment variables MUST USE `serial_test::serial`.
//!   This is because different tests reading different .env files causes a
//!   data race.
//!
//! - Persistent storage can be used in repository tests. Use randomized values
//!   for fields that may have unique constraints.
//!
//! - Use `#[ignored]` for tests that require a live production database or an
//!   external client, and write tests within examples for those that require
//!   human interaction.
