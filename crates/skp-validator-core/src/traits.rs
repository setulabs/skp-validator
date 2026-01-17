//! Core validation traits.
//!
//! This module defines the fundamental traits for the validation system:
//! - [`Validate`] - Main trait for validatable types
//! - [`Rule`] - Trait for individual validation rules
//! - [`Transform`] - Trait for field transformations
//! - [`ValidateNested`] - Trait for nested struct validation
//! - [`ValidateDive`] - Trait for collection validation

use crate::context::ValidationContext;
use crate::error::ValidationErrors;
use crate::path::FieldPath;
use crate::result::ValidationResult;

/// Core validation trait for types that can be validated.
///
/// This trait is typically implemented by the derive macro, but can also
/// be implemented manually for custom validation logic.
///
/// # Example
///
/// ```rust,ignore
/// use skp_validator_core::{Validate, ValidationContext, ValidationResult};
///
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// impl Validate for User {
///     fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
///         let mut errors = ValidationErrors::new();
///
///         if self.name.is_empty() {
///             errors.add_field_error("name",
///                 ValidationError::new("name", "required", "Name is required"));
///         }
///
///         if errors.is_empty() { Ok(()) } else { Err(errors) }
///     }
/// }
/// ```
pub trait Validate {
    /// Validate this instance with default context.
    fn validate(&self) -> ValidationResult<()> {
        self.validate_with_context(&ValidationContext::default())
    }

    /// Validate this instance with a custom context.
    ///
    /// This is the primary validation method that should be implemented.
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()>;

    /// Validate and return a transformed copy (if transformations are applied).
    ///
    /// This method is useful when validation includes transformations like
    /// trimming, case conversion, etc.
    fn validate_and_transform(&self) -> ValidationResult<Self>
    where
        Self: Clone + Sized,
    {
        self.validate()?;
        Ok(self.clone())
    }
}

// === Blanket Implementations ===

impl<T: Validate + ?Sized> Validate for Box<T> {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        (**self).validate_with_context(ctx)
    }
}

impl<T: Validate + ?Sized> Validate for std::rc::Rc<T> {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        (**self).validate_with_context(ctx)
    }
}

impl<T: Validate + ?Sized> Validate for std::sync::Arc<T> {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        (**self).validate_with_context(ctx)
    }
}

impl<T: Validate + ?Sized> Validate for &T {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        (**self).validate_with_context(ctx)
    }
}

/// Trait for individual validation rules.
///
/// Each validation rule (email, length, range, etc.) implements this trait.
/// Rules are generic over the value type they validate.
///
/// # Example
///
/// ```rust,ignore
/// use skp_validator_core::{Rule, ValidationContext, ValidationResult};
///
/// struct MinLengthRule {
///     min: usize,
/// }
///
/// impl Rule<str> for MinLengthRule {
///     fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
///         if value.len() >= self.min {
///             Ok(())
///         } else {
///             Err(ValidationErrors::from_error(
///                 ValidationError::new("", "length.min", "Too short")
///             ))
///         }
///     }
///
///     fn name(&self) -> &'static str { "min_length" }
///     fn default_message(&self) -> String { format!("Must be at least {} characters", self.min) }
/// }
/// ```
pub trait Rule<T: ?Sized>: Send + Sync {
    /// Validate the value.
    fn validate(&self, value: &T, ctx: &ValidationContext) -> ValidationResult<()>;

    /// Validate the value with a field path for error reporting.
    fn validate_at(
        &self,
        value: &T,
        path: &FieldPath,
        ctx: &ValidationContext,
    ) -> ValidationResult<()> {
        self.validate(value, ctx).map_err(|mut errors| {
            // Prepend path to all errors
            for error in &mut errors.errors {
                let mut new_path = path.clone();
                for segment in error.path.segments().iter().cloned() {
                    match segment {
                        crate::path::PathSegment::Field(f) => new_path.append_field(f),
                        crate::path::PathSegment::Index(i) => new_path.append_index(i),
                        crate::path::PathSegment::Key(k) => new_path.append_key(k),
                    }
                }
                error.path = new_path;
            }
            errors
        })
    }

    /// Get the rule name for error reporting.
    fn name(&self) -> &'static str;

    /// Get the default error message.
    fn default_message(&self) -> String;

    /// Get the error code.
    fn error_code(&self) -> String {
        self.name().to_string()
    }

    /// Check if this rule is a transformation (not validation).
    fn is_transform(&self) -> bool {
        false
    }
}

/// Trait for field transformations.
///
/// Transformations modify values during validation (e.g., trim, uppercase).
pub trait Transform<T>: Send + Sync {
    /// Transform the value.
    fn transform(&self, value: T) -> T;

    /// Get the transformation name.
    fn name(&self) -> &'static str;
}

/// Trait for nested struct validation.
///
/// Implemented by types that contain nested validatable structures.
pub trait ValidateNested: Validate {
    /// Validate as a nested field with path prefix.
    fn validate_nested(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()>;
}

/// Trait for collection validation (dive).
///
/// Implemented by collection types (Vec, HashMap, etc.) to validate each item.
pub trait ValidateDive<T: Validate> {
    /// Validate each item in the collection.
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()>;
}

// === Default implementations ===

impl<T: Validate> ValidateNested for T {
    fn validate_nested(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        self.validate_with_context(ctx).map_err(|errors| {
            let mut nested = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                nested.add_nested_errors(field, errors);
            } else {
                nested.merge(errors);
            }
            nested
        })
    }
}

impl<T: Validate> ValidateDive<T> for Vec<T> {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = std::collections::BTreeMap::new();

        for (idx, item) in self.iter().enumerate() {
            if let Err(item_errors) = item.validate_with_context(ctx) {
                errors.insert(idx, Box::new(item_errors));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut result = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                result.add_list_errors(field, errors);
            }
            Err(result)
        }
    }
}

impl<T: Validate> ValidateDive<T> for Option<T> {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        if let Some(value) = self {
            value.validate_nested(path, ctx)
        } else {
            Ok(())
        }
    }
}

impl<K, V> ValidateDive<V> for std::collections::HashMap<K, V>
where
    K: std::fmt::Display + Eq + std::hash::Hash,
    V: Validate,
{
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = std::collections::BTreeMap::new();

        for (key, value) in self.iter() {
            if let Err(item_errors) = value.validate_with_context(ctx) {
                errors.insert(key.to_string(), Box::new(item_errors));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut result = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                result.add_map_errors(field, errors);
            }
            Err(result)
        }
    }
}

impl<K, V> ValidateDive<V> for std::collections::BTreeMap<K, V>
where
    K: std::fmt::Display + Ord,
    V: Validate,
{
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = std::collections::BTreeMap::new();

        for (key, value) in self.iter() {
            if let Err(item_errors) = value.validate_with_context(ctx) {
                errors.insert(key.to_string(), Box::new(item_errors));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut result = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                result.add_map_errors(field, errors);
            }
            Err(result)
        }
    }
}

impl<T: Validate + Eq + std::hash::Hash> ValidateDive<T> for std::collections::HashSet<T> {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = std::collections::BTreeMap::new();

        for (idx, item) in self.iter().enumerate() {
            if let Err(item_errors) = item.validate_with_context(ctx) {
                errors.insert(idx, Box::new(item_errors));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut result = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                result.add_list_errors(field, errors);
            }
            Err(result)
        }
    }
}

impl<T: Validate + Ord> ValidateDive<T> for std::collections::BTreeSet<T> {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = std::collections::BTreeMap::new();

        for (idx, item) in self.iter().enumerate() {
            if let Err(item_errors) = item.validate_with_context(ctx) {
                errors.insert(idx, Box::new(item_errors));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut result = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                result.add_list_errors(field, errors);
            }
            Err(result)
        }
    }
}

// Smart pointer implementations - these dive into the inner type
impl<T: Validate> ValidateDive<T> for Box<T> {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        self.as_ref().validate_nested(path, ctx)
    }
}

impl<T: Validate> ValidateDive<T> for std::rc::Rc<T> {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        self.as_ref().validate_nested(path, ctx)
    }
}

impl<T: Validate> ValidateDive<T> for std::sync::Arc<T> {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        self.as_ref().validate_nested(path, ctx)
    }
}

// Slice implementation
impl<T: Validate> ValidateDive<T> for [T] {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = std::collections::BTreeMap::new();

        for (idx, item) in self.iter().enumerate() {
            if let Err(item_errors) = item.validate_with_context(ctx) {
                errors.insert(idx, Box::new(item_errors));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut result = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                result.add_list_errors(field, errors);
            }
            Err(result)
        }
    }
}

// Array implementations for common sizes
impl<T: Validate, const N: usize> ValidateDive<T> for [T; N] {
    fn validate_dive(&self, path: &FieldPath, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = std::collections::BTreeMap::new();

        for (idx, item) in self.iter().enumerate() {
            if let Err(item_errors) = item.validate_with_context(ctx) {
                errors.insert(idx, Box::new(item_errors));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut result = ValidationErrors::new();
            if let Some(field) = path.last_field_name() {
                result.add_list_errors(field, errors);
            }
            Err(result)
        }
    }
}

/// A composite rule that combines multiple rules.
pub struct CompositeRule<T: ?Sized> {
    rules: Vec<Box<dyn Rule<T>>>,
}

impl<T: ?Sized> CompositeRule<T> {
    /// Create a new composite rule.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to this composite.
    pub fn add(mut self, rule: impl Rule<T> + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }
}

impl<T: ?Sized> Default for CompositeRule<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Sync> Rule<T> for CompositeRule<T> {
    fn validate(&self, value: &T, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut all_errors = ValidationErrors::new();

        for rule in &self.rules {
            if let Err(errors) = rule.validate(value, ctx) {
                all_errors.merge(errors);

                // Fail fast if enabled
                if ctx.is_fail_fast() && !all_errors.is_empty() {
                    return Err(all_errors);
                }
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    fn name(&self) -> &'static str {
        "composite"
    }

    fn default_message(&self) -> String {
        "Validation failed".to_string()
    }
}

/// Any rule - passes if any of the inner rules pass (OR logic).
pub struct AnyRule<T: ?Sized> {
    rules: Vec<Box<dyn Rule<T>>>,
}

impl<T: ?Sized> AnyRule<T> {
    /// Create a new any rule.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule.
    pub fn add(mut self, rule: impl Rule<T> + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }
}

impl<T: ?Sized> Default for AnyRule<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Sync> Rule<T> for AnyRule<T> {
    fn validate(&self, value: &T, ctx: &ValidationContext) -> ValidationResult<()> {
        if self.rules.is_empty() {
            return Ok(());
        }

        for rule in &self.rules {
            if rule.validate(value, ctx).is_ok() {
                return Ok(()); // At least one passed
            }
        }

        // None passed, return generic error
        Err(ValidationErrors::from_iter([crate::ValidationError::root(
            "any",
            self.default_message(),
        )]))
    }

    fn name(&self) -> &'static str {
        "any"
    }

    fn default_message(&self) -> String {
        "Must satisfy at least one validation rule".to_string()
    }
}

/// All rule - passes only if all inner rules pass (AND logic, same as CompositeRule).
pub type AllRule<T> = CompositeRule<T>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValidationError;

    // Mock rule for testing
    struct MockRule {
        should_pass: bool,
    }

    impl Rule<str> for MockRule {
        fn validate(&self, _value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
            if self.should_pass {
                Ok(())
            } else {
                Err(ValidationErrors::from_iter([ValidationError::new(
                    "",
                    "mock",
                    "Mock error",
                )]))
            }
        }

        fn name(&self) -> &'static str {
            "mock"
        }

        fn default_message(&self) -> String {
            "Mock error".to_string()
        }
    }

    #[test]
    fn test_composite_rule() {
        let rule = CompositeRule::new()
            .add(MockRule { should_pass: true })
            .add(MockRule { should_pass: true });

        let ctx = ValidationContext::default();
        assert!(rule.validate("test", &ctx).is_ok());
    }

    #[test]
    fn test_composite_rule_fails() {
        let rule = CompositeRule::new()
            .add(MockRule { should_pass: true })
            .add(MockRule { should_pass: false });

        let ctx = ValidationContext::default();
        assert!(rule.validate("test", &ctx).is_err());
    }

    #[test]
    fn test_any_rule() {
        let rule = AnyRule::new()
            .add(MockRule { should_pass: false })
            .add(MockRule { should_pass: true });

        let ctx = ValidationContext::default();
        assert!(rule.validate("test", &ctx).is_ok());
    }

    #[test]
    fn test_any_rule_all_fail() {
        let rule = AnyRule::new()
            .add(MockRule { should_pass: false })
            .add(MockRule { should_pass: false });

        let ctx = ValidationContext::default();
        assert!(rule.validate("test", &ctx).is_err());
    }
}
