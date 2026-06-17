//! # metw-accounts-center
//!
//! metw-accounts-center is an identity management system implemented as a
//! zero-knowledge service.
//!
//! Goals:
//! - Authentication Service: An application can use this service as an
//!   authentication authority.
//! - End-to-End Encryption (E2EE): Users have a public-private key pair
//!   and a master key. Their master keys and private keys are stored on the
//!   server, encrypted on the client side.
//!
//! Non-Goals:
//! - This microservice must not be extended with non-authentication features;
//!   profiles, posts, shares, etc., are out of scope for this service.
//!
//!
//! ## The Ultimate Goal: E2EE-Authentication
//!
//! Account public keys are served publicly. When users log into their accounts,
//! they are essentially retrieving their own private keys. This private key
//! is used for OAuth2-like flows, where the users themselves authorize other
//! applications.
//!
//! ```txt
//! +-- CLIENT - (Account Creation) --------------------------------+
//! |                                                               |
//! |     User credentials                      Key encryption key  |
//! |    +------------------+  Key stretching  +-----+              |
//! |  --- password ---------------------------> KEK |              |
//! |  | |                  |                  +--|--+              |
//! |  | | client generated |                     | Encrypt         |
//! |  | | random keys:     |                  +--v-----------+     |
//! |  | | - master key -----------------------> encrypted MK |     |
//! |  | | - private key ----------------------> encrypted PK |     |
//! |  | | - public key ---------------------  +--------------+     |
//! |  | +------------------+               |         |             |
//! |  |                                    |         |             |
//! |  |  Argon2  +----------------------+  |         |             |
//! |  -----------> client password hash |  |         |             |
//! |             +----------------------+  |         |             |
//! |                       |               |         |             |
//! +-----------------------|---------------|---------|-------------+
//!                         |               |         |
//! +-- SERVER -------------|---------------|---------|-------------+
//! |                       |               |         |             |
//! |                 Authentication   Public Key   Key             |
//! |                 Storage          Registry     Vault           |
//! |                                                               |
//! +---------------------------------------------------------------+
//! ```
//!
//!
//! # Drafts
//!
//! The design principles of metw-accounts-center are specified in the drafts.
//! - [`acd_1`]: State Abstraction Layers
//! - [`acd_2`]: Writing Tests

/// # State Abstraction Layers
pub mod acd_1;

/// # Writing Tests
pub mod acd_2;
