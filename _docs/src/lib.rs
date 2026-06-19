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
//!   server, encrypted client-side.
//!
//! Non-Goals:
//! - This microservice must not be extended with non-authentication features:
//!   Profiles, posts, shares, and similar concepts are out of scope for this
//!   service.
//!
//!
//! ## Architecture: E2EE-Authentication
//!
//! Account public keys are served publicly. When users log into their accounts,
//! they retrieve their encrypted private keys from the server and decrypt them
//! locally. This private key is then used for OAuth2-like flows, where the
//! users directly authorize other applications.
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
//! # Setup
//!
//! Required environment variables for repository and client setup are listed
//! in fields of [`Config`](state::Config) struct.
//!
//! Once you have prepared the `.env` file, run the database migrations via:
//! ```sh
//! cd state/
//!
//! cargo sqlx migrate run
//! ```
//! (See the related
//! [sqlx-cli](https://github.com/transact-rs/sqlx/tree/main/sqlx-cli#create-and-run-migrations)
//! documentation)
//!
//! > **Important:** To ensure application security, carefully read the
//! > [Setup Recommendations](state#setup-recommendations)
//! > section.
//!
//!
//! # Drafts
//!
//! The design principles of metw-accounts-center are specified in the drafts.
//! - [`ACD-1`](acd_1): State Abstraction Layers
//! - [`ACD-2`](acd_2): Writing Tests

/// # State Abstraction Layers
pub mod acd_1;

/// # Writing Tests
pub mod acd_2;
