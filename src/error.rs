use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, Vec<ValidationError>>;

/// A validation error for a specific field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    /// The field name that failed validation
    pub field: String,
    /// The validation error message
    pub message: String,
    /// Optional additional metadata about the error
    pub metadata: Option<String>,
}

impl FieldError {
    /// Create a new field error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            metadata: None,
        }
    }

    /// Create a new field error with metadata
    pub fn with_metadata(
        field: impl Into<String>,
        message: impl Into<String>,
        metadata: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            metadata: Some(metadata.into()),
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(metadata) = &self.metadata {
            write!(f, "Field '{}': {} ({})", self.field, self.message, metadata)
        } else {
            write!(f, "Field '{}': {}", self.field, self.message)
        }
    }
}

/// Alias for FieldError for backward compatibility
pub type ValidationError = FieldError;

/// Error type for JSON validation operations
#[derive(Debug, Clone)]
pub struct JsonValidationError {
    /// The JSON path that failed validation
    pub path: String,
    /// The validation error message
    pub message: String,
    /// Optional additional metadata
    pub metadata: Option<String>,
}

impl JsonValidationError {
    /// Create a new JSON validation error
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            metadata: None,
        }
    }

    /// Create a new JSON validation error with metadata
    pub fn with_metadata(
        path: impl Into<String>,
        message: impl Into<String>,
        metadata: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            metadata: Some(metadata.into()),
        }
    }
}

impl fmt::Display for JsonValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(metadata) = &self.metadata {
            write!(
                f,
                "JSON path '{}': {} ({})",
                self.path, self.message, metadata
            )
        } else {
            write!(f, "JSON path '{}': {}", self.path, self.message)
        }
    }
}

impl std::error::Error for JsonValidationError {}
