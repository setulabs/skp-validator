//! # skp-validator-axum
//!
//! Axum integration for skp-validator.
//!
//! This crate provides seamless validation in Axum web applications
//! through a `ValidatedJson` extractor that automatically validates
//! incoming JSON payloads.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use axum::{routing::post, Router};
//! use skp_validator_axum::ValidatedJson;
//! use skp_validator::Validate;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Validate)]
//! struct CreateUser {
//!     #[validate(required, length(min = 3))]
//!     name: String,
//!     #[validate(email)]
//!     email: String,
//! }
//!
//! async fn create_user(ValidatedJson(user): ValidatedJson<CreateUser>) -> String {
//!     format!("Created user: {}", user.name)
//! }
//!
//! let app = Router::new().route("/users", post(create_user));
//! ```
//!
//! ## Custom Error Responses
//!
//! You can customize how validation errors are returned by implementing
//! `ValidationErrorResponse` for your error type.

mod extractor;
mod rejection;

pub use extractor::ValidatedJson;
pub use rejection::{ValidationRejection, JsonRejection};

/// Re-export core types for convenience
pub mod prelude {
    pub use crate::ValidatedJson;
    pub use crate::rejection::{ValidationRejection, JsonRejection};
    pub use skp_validator_core::{Validate, ValidationContext, ValidationErrors, ValidationError};
}
