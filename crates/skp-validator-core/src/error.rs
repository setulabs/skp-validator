//! Structured validation error types.
//!
//! Provides nested, path-aware validation errors that can represent:
//! - Field-level errors
//! - Nested struct errors
//! - Collection (array/map) item errors
//!
//! # Error Structure
//!
//! ```text
//! ValidationErrors
//! ├── errors: Vec<ValidationError>        // Root-level errors
//! └── fields: BTreeMap<String, FieldErrors>
//!     ├── Simple(Vec<ValidationError>)    // Field errors
//!     ├── Nested(Box<ValidationErrors>)   // Nested struct
//!     ├── List(BTreeMap<usize, ...>)      // Array items
//!     └── Map(BTreeMap<String, ...>)      // Map entries
//! ```

use crate::path::FieldPath;
use std::collections::BTreeMap;
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single validation error with path, code, and message.
///
/// # Example
///
/// ```rust
/// use skp_validator_core::{ValidationError, FieldPath};
///
/// let error = ValidationError::new("email", "email.invalid", "Must be a valid email address")
///     .with_param("value", "invalid-email");
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationError {
    /// Path to the field that failed validation
    pub path: FieldPath,

    /// Error code for programmatic handling (e.g., "email.invalid", "length.min")
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional parameters for error formatting and i18n
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub params: BTreeMap<String, ErrorParam>,
}

/// Parameter value for validation errors (supports multiple types)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum ErrorParam {
    /// String parameter
    String(String),
    /// Integer parameter
    Int(i64),
    /// Float parameter
    Float(f64),
    /// Boolean parameter
    Bool(bool),
    /// List of values
    List(Vec<String>),
}

impl From<String> for ErrorParam {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ErrorParam {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for ErrorParam {
    fn from(n: i64) -> Self {
        Self::Int(n)
    }
}

impl From<i32> for ErrorParam {
    fn from(n: i32) -> Self {
        Self::Int(n as i64)
    }
}

impl From<usize> for ErrorParam {
    fn from(n: usize) -> Self {
        Self::Int(n as i64)
    }
}

impl From<f64> for ErrorParam {
    fn from(n: f64) -> Self {
        Self::Float(n)
    }
}

impl From<bool> for ErrorParam {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Vec<String>> for ErrorParam {
    fn from(v: Vec<String>) -> Self {
        Self::List(v)
    }
}

impl ValidationError {
    /// Create a new validation error.
    ///
    /// # Arguments
    ///
    /// * `field` - The field name (will be converted to a FieldPath)
    /// * `code` - Error code for programmatic handling
    /// * `message` - Human-readable message
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: FieldPath::from_field(field),
            code: code.into(),
            message: message.into(),
            params: BTreeMap::new(),
        }
    }

    /// Create a new validation error with a full path.
    pub fn with_path(
        path: FieldPath,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path,
            code: code.into(),
            message: message.into(),
            params: BTreeMap::new(),
        }
    }

    /// Create a root-level error (empty path).
    pub fn root(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: FieldPath::new(),
            code: code.into(),
            message: message.into(),
            params: BTreeMap::new(),
        }
    }

    /// Add a parameter to this error.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<ErrorParam>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Get the field name (last segment of path)
    pub fn field_name(&self) -> Option<&str> {
        self.path.last_field_name()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.path, self.message)
        }
    }
}

impl std::error::Error for ValidationError {}

/// Container for field-level errors, supporting nested structures.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum FieldErrors {
    /// Simple list of errors for this field
    Simple(Vec<ValidationError>),

    /// Nested struct errors
    Nested(Box<ValidationErrors>),

    /// List/array item errors (index -> errors)
    List(BTreeMap<usize, Box<ValidationErrors>>),

    /// Map item errors (key -> errors)
    Map(BTreeMap<String, Box<ValidationErrors>>),
}

impl FieldErrors {
    /// Create simple field errors
    pub fn simple(errors: Vec<ValidationError>) -> Self {
        Self::Simple(errors)
    }

    /// Create nested struct errors
    pub fn nested(errors: ValidationErrors) -> Self {
        Self::Nested(Box::new(errors))
    }

    /// Create list errors
    pub fn list(errors: BTreeMap<usize, Box<ValidationErrors>>) -> Self {
        Self::List(errors)
    }

    /// Create map errors
    pub fn map(errors: BTreeMap<String, Box<ValidationErrors>>) -> Self {
        Self::Map(errors)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Simple(v) => v.is_empty(),
            Self::Nested(n) => n.is_empty(),
            Self::List(m) => m.is_empty(),
            Self::Map(m) => m.is_empty(),
        }
    }

    /// Count total errors recursively
    pub fn count(&self) -> usize {
        match self {
            Self::Simple(v) => v.len(),
            Self::Nested(n) => n.count(),
            Self::List(m) => m.values().map(|v| v.count()).sum(),
            Self::Map(m) => m.values().map(|v| v.count()).sum(),
        }
    }
}

/// Container for validation errors with nested structure support.
///
/// # Example
///
/// ```rust
/// use skp_validator_core::{ValidationErrors, ValidationError};
///
/// let mut errors = ValidationErrors::new();
/// errors.add_field_error("email", ValidationError::new("email", "email.invalid", "Invalid email"));
/// errors.add_field_error("name", ValidationError::new("name", "required", "Name is required"));
///
/// assert_eq!(errors.count(), 2);
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationErrors {
    /// Root-level errors (struct-wide validations)
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub errors: Vec<ValidationError>,

    /// Field-level errors
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub fields: BTreeMap<String, FieldErrors>,
}

impl ValidationErrors {
    /// Create an empty error container
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a single error
    pub fn from_error(error: ValidationError) -> Self {
        let mut errors = Self::new();
        if let Some(field) = error.field_name().map(|s| s.to_string()) {
            errors.add_field_error(field, error);
        } else {
            errors.add_root_error(error);
        }
        errors
    }

    /// Check if there are no errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.fields.is_empty()
    }

    /// Count total errors recursively
    pub fn count(&self) -> usize {
        self.errors.len() + self.fields.values().map(|f| f.count()).sum::<usize>()
    }

    /// Add a root-level error (not associated with a specific field)
    pub fn add_root_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a field-level error
    pub fn add_field_error(&mut self, field: impl Into<String>, error: ValidationError) {
        let field = field.into();
        match self.fields.entry(field) {
            std::collections::btree_map::Entry::Occupied(mut e) => {
                if let FieldErrors::Simple(vec) = e.get_mut() {
                    vec.push(error);
                }
            }
            std::collections::btree_map::Entry::Vacant(e) => {
                e.insert(FieldErrors::Simple(vec![error]));
            }
        }
    }

    /// Add nested struct errors
    pub fn add_nested_errors(&mut self, field: impl Into<String>, errors: ValidationErrors) {
        if !errors.is_empty() {
            self.fields
                .insert(field.into(), FieldErrors::Nested(Box::new(errors)));
        }
    }

    /// Add list item errors
    pub fn add_list_errors(
        &mut self,
        field: impl Into<String>,
        errors: BTreeMap<usize, Box<ValidationErrors>>,
    ) {
        if !errors.is_empty() {
            self.fields.insert(field.into(), FieldErrors::List(errors));
        }
    }

    /// Add map item errors
    pub fn add_map_errors(
        &mut self,
        field: impl Into<String>,
        errors: BTreeMap<String, Box<ValidationErrors>>,
    ) {
        if !errors.is_empty() {
            self.fields.insert(field.into(), FieldErrors::Map(errors));
        }
    }

    /// Merge another ValidationErrors into this one
    pub fn merge(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
        for (field, errors) in other.fields {
            self.fields.insert(field, errors);
        }
    }

    /// Merge errors from a Result
    pub fn merge_result<T>(&mut self, result: Result<T, ValidationErrors>) -> Option<T> {
        match result {
            Ok(v) => Some(v),
            Err(e) => {
                self.merge(e);
                None
            }
        }
    }

    /// Convert to a flat map format: "path" -> ["error1", "error2"]
    ///
    /// Useful for simpler error handling or compatibility with other formats.
    pub fn to_flat_map(&self) -> BTreeMap<String, Vec<String>> {
        let mut result = BTreeMap::new();
        self.flatten_into("", &mut result);
        result
    }

    fn flatten_into(&self, prefix: &str, result: &mut BTreeMap<String, Vec<String>>) {
        // Add root errors
        for error in &self.errors {
            let key = if prefix.is_empty() {
                "_root".to_string()
            } else {
                prefix.to_string()
            };
            result.entry(key).or_default().push(error.message.clone());
        }

        // Add field errors
        for (field, errors) in &self.fields {
            let path = if prefix.is_empty() {
                field.clone()
            } else {
                format!("{}.{}", prefix, field)
            };

            match errors {
                FieldErrors::Simple(vec) => {
                    for error in vec {
                        result.entry(path.clone()).or_default().push(error.message.clone());
                    }
                }
                FieldErrors::Nested(nested) => {
                    nested.flatten_into(&path, result);
                }
                FieldErrors::List(list) => {
                    for (idx, nested) in list {
                        let item_path = format!("{}[{}]", path, idx);
                        nested.flatten_into(&item_path, result);
                    }
                }
                FieldErrors::Map(map) => {
                    for (key, nested) in map {
                        let item_path = format!("{}[{}]", path, key);
                        nested.flatten_into(&item_path, result);
                    }
                }
            }
        }
    }

    /// Get all error messages as a flat list
    pub fn messages(&self) -> Vec<String> {
        let flat = self.to_flat_map();
        flat.into_values().flatten().collect()
    }

    /// Get errors for a specific field
    pub fn field(&self, name: &str) -> Option<&FieldErrors> {
        self.fields.get(name)
    }

    /// Check if a specific field has errors
    pub fn has_field_error(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// Get the number of fields with errors
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Iterate over field errors
    pub fn field_errors(&self) -> impl Iterator<Item = (&String, &FieldErrors)> {
        self.fields.iter()
    }

    /// Convert to JSON value (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Convert to JSON string (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Convert to pretty JSON string (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

impl std::error::Error for ValidationErrors {}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flat = self.to_flat_map();
        let mut first = true;
        for (field, messages) in flat {
            for msg in messages {
                if !first {
                    writeln!(f)?;
                }
                first = false;
                if field == "_root" {
                    write!(f, "{}", msg)?;
                } else {
                    write!(f, "{}: {}", field, msg)?;
                }
            }
        }
        Ok(())
    }
}

impl FromIterator<ValidationError> for ValidationErrors {
    fn from_iter<I: IntoIterator<Item = ValidationError>>(iter: I) -> Self {
        let mut errors = ValidationErrors::new();
        for error in iter {
            if let Some(field) = error.field_name() {
                errors.add_field_error(field.to_string(), error);
            } else {
                errors.add_root_error(error);
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_error() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(
            "email",
            ValidationError::new("email", "email.invalid", "Invalid email"),
        );

        assert!(!errors.is_empty());
        assert_eq!(errors.count(), 1);
        assert!(errors.has_field_error("email"));
    }

    #[test]
    fn test_multiple_errors() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(
            "email",
            ValidationError::new("email", "email.invalid", "Invalid email"),
        );
        errors.add_field_error(
            "name",
            ValidationError::new("name", "required", "Name is required"),
        );

        assert_eq!(errors.count(), 2);
    }

    #[test]
    fn test_flat_map() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(
            "email",
            ValidationError::new("email", "email.invalid", "Invalid email"),
        );

        let flat = errors.to_flat_map();
        assert!(flat.contains_key("email"));
        assert_eq!(flat["email"][0], "Invalid email");
    }

    #[test]
    fn test_nested_errors() {
        let mut inner = ValidationErrors::new();
        inner.add_field_error(
            "city",
            ValidationError::new("city", "required", "City is required"),
        );

        let mut outer = ValidationErrors::new();
        outer.add_nested_errors("address", inner);

        assert_eq!(outer.count(), 1);
        let flat = outer.to_flat_map();
        assert!(flat.contains_key("address.city"));
    }
}
