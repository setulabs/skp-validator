//! Validation context for runtime validation state.
//!
//! The [`ValidationContext`] provides:
//! - Access to field values during validation
//! - Metadata storage for cross-field validation
//! - Configuration options for validation behavior

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "serde")]
use serde_json::Value as JsonValue;

/// Context for validation operations.
///
/// Provides runtime state during validation, including:
/// - Field values for cross-field validation
/// - Metadata storage
/// - Locale settings for i18n
///
/// # Example
///
/// ```rust
/// use skp_validator_core::ValidationContext;
///
/// let ctx = ValidationContext::new()
///     .with_locale("en")
///     .with_meta("request_id", "123");
/// ```
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    /// Field values for cross-field validation (JSON values)
    #[cfg(feature = "serde")]
    field_values: HashMap<String, JsonValue>,

    /// Field values without serde (using string representation)
    #[cfg(not(feature = "serde"))]
    field_values: HashMap<String, String>,

    /// Arbitrary metadata
    metadata: HashMap<String, String>,

    /// Locale for error messages (default: "en")
    locale: String,

    /// Whether to collect all errors or fail fast
    fail_fast: bool,

    /// Custom data (type-erased)
    custom_data: Option<Arc<dyn Any + Send + Sync>>,
}

impl ValidationContext {
    /// Create a new empty validation context
    pub fn new() -> Self {
        Self {
            field_values: HashMap::new(),
            metadata: HashMap::new(),
            locale: "en".to_string(),
            fail_fast: false,
            custom_data: None,
        }
    }

    /// Set the locale for error messages
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Set fail-fast mode (stop on first error)
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Add metadata
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set custom data (type-erased)
    pub fn with_custom_data<T: Any + Send + Sync>(mut self, data: T) -> Self {
        self.custom_data = Some(Arc::new(data));
        self
    }

    /// Get the locale
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Check if fail-fast mode is enabled
    pub fn is_fail_fast(&self) -> bool {
        self.fail_fast
    }

    /// Get metadata value
    pub fn get_meta(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Get custom data by type
    pub fn get_custom_data<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.custom_data
            .as_ref()
            .and_then(|d| d.downcast_ref::<T>())
    }

    // === Field value access (with serde feature) ===

    /// Create context from a JSON value (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn from_json(json: &JsonValue) -> Self {
        let mut field_values = HashMap::new();

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                field_values.insert(key.clone(), value.clone());
            }
        }

        Self {
            field_values,
            metadata: HashMap::new(),
            locale: "en".to_string(),
            fail_fast: false,
            custom_data: None,
        }
    }

    /// Create context from a serializable object (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn from_serde<T: serde::Serialize>(data: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_value(data)?;
        Ok(Self::from_json(&json))
    }

    /// Get a field value as JSON (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn get_field(&self, name: &str) -> Option<&JsonValue> {
        self.field_values.get(name)
    }

    /// Set a field value (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn set_field(&mut self, name: impl Into<String>, value: JsonValue) {
        self.field_values.insert(name.into(), value);
    }

    /// Set a field value (builder pattern, requires serde feature)
    #[cfg(feature = "serde")]
    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.field_values.insert(name.into(), JsonValue::String(value.into()));
        self
    }

    /// Get a field value as string (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get_field(name)?.as_str()
    }

    /// Get a field value as i64 (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn get_i64(&self, name: &str) -> Option<i64> {
        self.get_field(name)?.as_i64()
    }

    /// Get a field value as f64 (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn get_f64(&self, name: &str) -> Option<f64> {
        self.get_field(name)?.as_f64()
    }

    /// Get a field value as bool (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get_field(name)?.as_bool()
    }

    /// Check if a field exists and is not null/empty (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn has_value(&self, name: &str) -> bool {
        if let Some(value) = self.get_field(name) {
            !value.is_null()
                && match value {
                    JsonValue::String(s) => !s.trim().is_empty(),
                    JsonValue::Array(arr) => !arr.is_empty(),
                    JsonValue::Object(obj) => !obj.is_empty(),
                    _ => true,
                }
        } else {
            false
        }
    }

    /// Check if a field is empty or null (requires serde feature)
    #[cfg(feature = "serde")]
    pub fn is_empty(&self, name: &str) -> bool {
        !self.has_value(name)
    }

    /// Get all field names
    pub fn field_names(&self) -> impl Iterator<Item = &String> {
        self.field_values.keys()
    }

    // === Field value access (without serde feature) ===

    /// Set a field value as string (no serde)
    #[cfg(not(feature = "serde"))]
    pub fn set_field(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.field_values.insert(name.into(), value.into());
    }

    /// Get a field value as string (no serde)
    #[cfg(not(feature = "serde"))]
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.field_values.get(name).map(|s| s.as_str())
    }

    /// Check if a field exists (no serde)
    #[cfg(not(feature = "serde"))]
    pub fn has_value(&self, name: &str) -> bool {
        self.field_values.get(name).map(|s| !s.is_empty()).unwrap_or(false)
    }
}

/// Builder for ValidationContext with custom context type
pub struct ValidationContextBuilder<C> {
    context: ValidationContext,
    custom: Option<C>,
}

impl<C: Any + Send + Sync> ValidationContextBuilder<C> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            context: ValidationContext::new(),
            custom: None,
        }
    }

    /// Set the locale
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.context.locale = locale.into();
        self
    }

    /// Set fail-fast mode
    pub fn fail_fast(mut self, fail_fast: bool) -> Self {
        self.context.fail_fast = fail_fast;
        self
    }

    /// Add metadata
    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.metadata.insert(key.into(), value.into());
        self
    }

    /// Set custom context data
    pub fn custom(mut self, custom: C) -> Self {
        self.custom = Some(custom);
        self
    }

    /// Build the context
    pub fn build(mut self) -> ValidationContext {
        if let Some(custom) = self.custom {
            self.context.custom_data = Some(Arc::new(custom));
        }
        self.context
    }
}

impl<C: Any + Send + Sync> Default for ValidationContextBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ValidationContext::new()
            .with_locale("hi")
            .with_meta("request_id", "123");

        assert_eq!(ctx.locale(), "hi");
        assert_eq!(ctx.get_meta("request_id"), Some("123"));
    }

    #[test]
    fn test_custom_data() {
        #[derive(Debug, Clone)]
        struct MyContext {
            user_id: u64,
        }

        let ctx = ValidationContext::new().with_custom_data(MyContext { user_id: 42 });

        let my_ctx = ctx.get_custom_data::<MyContext>().unwrap();
        assert_eq!(my_ctx.user_id, 42);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_from_json() {
        let json = serde_json::json!({
            "name": "John",
            "age": 30,
            "active": true
        });

        let ctx = ValidationContext::from_json(&json);

        assert_eq!(ctx.get_string("name"), Some("John"));
        assert_eq!(ctx.get_i64("age"), Some(30));
        assert_eq!(ctx.get_bool("active"), Some(true));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_has_value() {
        let json = serde_json::json!({
            "name": "John",
            "empty": "",
            "null": null
        });

        let ctx = ValidationContext::from_json(&json);

        assert!(ctx.has_value("name"));
        assert!(!ctx.has_value("empty"));
        assert!(!ctx.has_value("null"));
        assert!(!ctx.has_value("missing"));
    }
}
