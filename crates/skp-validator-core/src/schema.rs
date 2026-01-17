//! Schema introspection types.

use std::collections::BTreeMap;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Metadata about the validation rules on a type.
pub trait ValidationMetadata {
    /// Get the validation rules for this type.
    fn get_validation_rules() -> TypeValidation;
}

/// Description of validation rules for a type.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeValidation {
    /// Rules for each field.
    pub fields: BTreeMap<String, FieldValidation>,
    /// Nested types that also have validation.
    pub nested: BTreeMap<String, TypeValidation>,
}

/// Description of validation rules for a single field.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldValidation {
    /// The validation rules applied to this field.
    pub rules: Vec<RuleSchema>,
}

/// Schema description of a single validation rule.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "params"))]
pub enum RuleSchema {
    /// String length
    Length { min: Option<u64>, max: Option<u64>, equal: Option<u64> },
    /// Email format
    Email,
    /// URL format
    Url,
    /// IP address
    Ip { version: Option<String> },
    /// UUID
    Uuid { version: Option<usize> },
    /// Phone number
    Phone,
    /// Credit card
    CreditCard,
    /// Regex pattern
    Pattern { regex: String },
    /// Numeric range
    Range { min: Option<f64>, max: Option<f64>, min_exclusive: Option<f64>, max_exclusive: Option<f64> },
    /// Allow values
    AllowedValues { values: Vec<String> },
    /// Must match another field
    MustMatch { other_field: String },
    /// Required field
    Required,
    /// Custom validation
    Custom { name: String },
}

impl TypeValidation {
    /// Create a new TypeValidation
    pub fn new() -> Self {
        Self::default()
    }
}

mod impls;
