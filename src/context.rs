use serde_json::Value;
use std::collections::HashMap;

/// Context for validation operations, providing access to field values
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// All field values in the current validation context
    pub field_values: HashMap<String, Value>,
    /// Optional additional metadata
    pub metadata: HashMap<String, Value>,
}

impl ValidationContext {
    /// Create a new validation context from a JSON value
    pub fn from_json(json: &Value) -> Self {
        let mut field_values = HashMap::new();

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                field_values.insert(key.clone(), value.clone());
            }
        }

        Self {
            field_values,
            metadata: HashMap::new(),
        }
    }

    /// Create a new validation context from a serializable object
    pub fn from_serde<T: serde::Serialize>(data: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_value(data)?;
        Ok(Self::from_json(&json))
    }

    /// Create a new empty validation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a field value by name
    pub fn get_field_value(&self, field_name: &str) -> Option<&Value> {
        self.field_values.get(field_name)
    }

    /// Get a field value as a string
    pub fn get_string(&self, field_name: &str) -> Option<&str> {
        self.get_field_value(field_name)?.as_str()
    }

    /// Get a field value as a number
    pub fn get_number(&self, field_name: &str) -> Option<f64> {
        self.get_field_value(field_name)?.as_f64()
    }

    /// Get a field value as an integer
    pub fn get_int(&self, field_name: &str) -> Option<i64> {
        self.get_field_value(field_name)?.as_i64()
    }

    /// Get a field value as a boolean
    pub fn get_bool(&self, field_name: &str) -> Option<bool> {
        self.get_field_value(field_name)?.as_bool()
    }

    /// Check if a field has a non-null, non-empty value
    pub fn has_value(&self, field_name: &str) -> bool {
        if let Some(value) = self.get_field_value(field_name) {
            !value.is_null()
                && match value {
                    Value::String(s) => !s.trim().is_empty(),
                    Value::Array(arr) => !arr.is_empty(),
                    Value::Object(obj) => !obj.is_empty(),
                    _ => true,
                }
        } else {
            false
        }
    }

    /// Check if a field is empty or null
    pub fn is_empty(&self, field_name: &str) -> bool {
        !self.has_value(field_name)
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Get all field names
    pub fn field_names(&self) -> Vec<&String> {
        self.field_values.keys().collect()
    }

    /// Set or update a field value
    pub fn set_field_value(&mut self, field_name: &str, value: Value) {
        self.field_values.insert(field_name.to_string(), value);
    }

    /// Create a sub-context with only specific fields
    pub fn sub_context(&self, field_names: &[&str]) -> Self {
        let mut field_values = HashMap::new();
        for &field_name in field_names {
            if let Some(value) = self.field_values.get(field_name) {
                field_values.insert(field_name.to_string(), value.clone());
            }
        }

        Self {
            field_values,
            metadata: self.metadata.clone(),
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            field_values: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}
