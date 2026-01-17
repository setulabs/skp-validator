//! # skp-validator-core
//!
//! Core traits and types for the skp-validator validation library.
//!
//! This crate provides the foundational building blocks:
//! - [`Validate`] - Core trait for validatable types
//! - [`Rule`] - Trait for individual validation rules
//! - [`Transform`] - Trait for field transformations
//! - [`ValidationErrors`] - Structured, nested error container
//! - [`ValidationContext`] - Runtime context for validation
//!
//! ## Example
//!
//! ```rust,ignore
//! use skp_validator_core::{Validate, ValidationResult};
//!
//! struct User {
//!     name: String,
//!     email: String,
//! }
//!
//! impl Validate for User {
//!     fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
//!         // Validation logic here
//!         Ok(())
//!     }
//! }
//! ```

mod context;
mod error;
mod path;
mod result;
pub mod schema;
mod traits;

pub use context::*;
pub use error::*;
pub use path::*;
pub use result::*;
pub use traits::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::context::ValidationContext;
    pub use crate::error::{FieldErrors, ValidationError, ValidationErrors};
    pub use crate::path::{FieldPath, PathSegment};
    pub use crate::result::ValidationResult;
    pub use crate::traits::{Rule, Transform, Validate, ValidateDive, ValidateNested};
}
