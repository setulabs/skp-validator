//! # OneClick Validator
//!
//! A comprehensive, reusable validation framework for Rust structs and JSON objects.
//! Provides declarative validation rules without hardcoding or service-specific logic.
//!
//! ## Features
//!
//! - **Declarative validation**: Use derive macros and attributes on your structs
//! - **Multiple validation types**: Required, length, regex, numeric ranges, dates, custom validators
//! - **Contextual validation**: Rules that depend on other field values
//! - **JSON support**: Validate JSON objects directly
//! - **Custom error messages**: Detailed validation error reporting
//! - **Zero dependencies on external services**: Pure validation logic
//!
//! ## Quick Start
//!
//! ```rust
//! use oc_validator::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
//! struct User {
//!     #[validate(required, length(min = 3, max = 50))]
//!     pub name: String,
//!
//!     #[validate(required, email)]
//!     pub email: String,
//!
//!     #[validate(range(min = 18, max = 120))]
//!     pub age: Option<u32>,
//!
//!     #[validate(required, regex(path = "PHONE_REGEX"))]
//!     pub phone: Option<String>,
//! }
//!
//! // Validate a struct
//! let user = User {
//!     name: "John".to_string(),
//!     email: "john@example.com".to_string(),
//!     age: Some(25),
//!     phone: Some("+1234567890".to_string()),
//! };
//!
//! match user.validate() {
//!     Ok(_) => println!("User is valid!"),
//!     Err(errors) => {
//!         for error in errors {
//!             println!("Validation error: {}", error);
//!         }
//!     }
//! }
//! ```

pub mod context;
pub mod error;
pub mod json_validator;
pub mod rules;
pub mod validator;

pub use context::*;
pub use error::*;
pub use json_validator::validate_json;
pub use rules::*;
pub use validator::{Validate, validate_json_str, validate_json_value, validate_object};

// Re-export derive macro
pub use oc_validator_macro::Validate;

/// Common imports for using OneClick Validator
pub mod prelude {
    pub use crate::FieldError;
    pub use crate::JsonValidationError;
    pub use crate::Validate;
    pub use crate::ValidationError;
    pub use crate::ValidationResult;
    pub use crate::validate_json;
}
