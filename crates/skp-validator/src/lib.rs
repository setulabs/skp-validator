//! # skp-validator
//!
//! The most advanced, flexible and modular validation library for Rust.
//!
//! ## Features
//!
//! - **Declarative Validation**: Use derive macros with powerful attributes
//! - **30+ Built-in Validators**: email, url, ip, uuid, phone, credit_card, etc.
//! - **Nested Validation**: Automatic validation of nested structs
//! - **Collection Validation**: Dive into Vec, HashMap, Option
//! - **Field Dependencies**: Conditional validation based on other fields
//! - **Field Transformations**: uppercase, lowercase, trim
//! - **Structured Errors**: Nested, JSON-serializable error format
//! - **Runtime JSON Validation**: Validate JSON without deserialization
//! - **i18n Support**: Localized error messages
//! - **Framework Adapters**: Axum, Actix integration
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use skp_validator::Validate;
//!
//! #[derive(Validate)]
//! struct User {
//!     #[validate(required, length(min = 3, max = 50))]
//!     name: String,
//!
//!     #[validate(required, email)]
//!     email: String,
//!
//!     #[validate(range(min = 18, max = 120))]
//!     age: Option<u32>,
//!
//!     #[validate(nested)]
//!     address: Address,
//!
//!     #[validate(dive, length(min = 1, max = 20))]
//!     tags: Vec<String>,
//! }
//!
//! let user = User { /* ... */ };
//! match user.validate() {
//!     Ok(()) => println!("Valid!"),
//!     Err(errors) => println!("Errors: {}", errors),
//! }
//! ```

// Re-export core types
pub use skp_validator_core::*;

// Re-export rules
pub use skp_validator_rules as rules;

// Re-export derive macro
#[cfg(feature = "derive")]
pub use skp_validator_derive::Validate;

/// Prelude for convenient imports
pub mod prelude {
    pub use skp_validator_core::prelude::*;

    #[cfg(feature = "derive")]
    pub use skp_validator_derive::Validate;
}
