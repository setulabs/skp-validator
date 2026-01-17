use crate::context::ValidationContext;
use crate::error::{FieldError, ValidationResult};
use crate::rules::ValidationConfig;
use serde::Serialize;

/// Trait for types that can be validated
pub trait Validate {
    /// Validate this instance and return any validation errors
    fn validate(&self) -> ValidationResult<()>;

    /// Validate this instance with field transformations applied
    fn validate_with_transform(&self) -> ValidationResult<Self>
    where
        Self: Clone + Sized;

    /// Get the validation configuration for this type
    fn validation_config() -> ValidationConfig;
}

/// Helper function to validate any serializable object
pub fn validate_object<T: Validate + Serialize>(data: &T) -> ValidationResult<()> {
    data.validate()
}

/// Helper function to validate a JSON string
pub fn validate_json_str<T: Validate>(json_str: &str) -> Result<T, Vec<FieldError>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let data: T = serde_json::from_str(json_str).map_err(|e| {
        vec![FieldError::new(
            "json",
            format!("JSON parsing error: {}", e),
        )]
    })?;

    data.validate()?;
    Ok(data)
}

/// Helper function to validate a JSON Value
pub fn validate_json_value<T: Validate>(
    json_value: &serde_json::Value,
) -> Result<T, Vec<FieldError>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let data: T = serde_json::from_value(json_value.clone()).map_err(|e| {
        vec![FieldError::new(
            "json",
            format!("JSON parsing error: {}", e),
        )]
    })?;

    data.validate()?;
    Ok(data)
}

/// Validate a JSON string directly without deserializing to a specific type
pub fn validate_json_with_config(
    json_str: &str,
    config: &ValidationConfig,
) -> ValidationResult<()> {
    let json_value: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        vec![FieldError::new(
            "json",
            format!("JSON parsing error: {}", e),
        )]
    })?;

    let mut context = ValidationContext::from_json(&json_value);
    config.validate(&mut context)
}

/// Validate a JSON Value directly with a validation config
pub fn validate_json_value_with_config(
    json_value: &serde_json::Value,
    config: &ValidationConfig,
) -> ValidationResult<()> {
    let mut context = ValidationContext::from_json(json_value);
    config.validate(&mut context)
}

/// Macro to implement Validate for a type with a given configuration
#[macro_export]
macro_rules! impl_validate {
    ($type:ty, $config:expr) => {
        impl $crate::Validate for $type {
            fn validate(&self) -> $crate::ValidationResult<()> {
                let mut context = $crate::ValidationContext::from_serde(self).map_err(|e| {
                    vec![$crate::FieldError::new(
                        "serialization",
                        format!("Serialization error: {}", e),
                    )]
                })?;
                Self::validation_config().validate(&mut context)
            }

            fn validation_config() -> $crate::ValidationConfig {
                $config
            }
        }
    };
}

/// Macro to create a validation rule
#[macro_export]
macro_rules! validation_rule {
    (required) => {
        $crate::ValidationRule::Required { message: None }
    };
    (required, message = $msg:expr) => {
        $crate::ValidationRule::Required {
            message: Some($msg.to_string()),
        }
    };
    (length(min = $min:expr)) => {
        $crate::ValidationRule::Length {
            min: Some($min),
            max: None,
            message: None,
        }
    };
    (length(max = $max:expr)) => {
        $crate::ValidationRule::Length {
            min: None,
            max: Some($max),
            message: None,
        }
    };
    (length(min = $min:expr, max = $max:expr)) => {
        $crate::ValidationRule::Length {
            min: Some($min),
            max: Some($max),
            message: None,
        }
    };
    (regex($pattern:expr)) => {
        $crate::ValidationRule::Regex {
            pattern: $pattern.to_string(),
            message: None,
        }
    };
    (email) => {
        $crate::ValidationRule::Email { message: None }
    };
    (phone) => {
        $crate::ValidationRule::Phone { message: None }
    };
    (range(min = $min:expr)) => {
        $crate::ValidationRule::Range {
            min: Some($min),
            max: None,
            message: None,
        }
    };
    (range(max = $max:expr)) => {
        $crate::ValidationRule::Range {
            min: None,
            max: Some($max),
            message: None,
        }
    };
    (range(min = $min:expr, max = $max:expr)) => {
        $crate::ValidationRule::Range {
            min: Some($min),
            max: Some($max),
            message: None,
        }
    };
    (int_range(min = $min:expr)) => {
        $crate::ValidationRule::IntRange {
            min: Some($min),
            max: None,
            message: None,
        }
    };
    (int_range(max = $max:expr)) => {
        $crate::ValidationRule::IntRange {
            min: None,
            max: Some($max),
            message: None,
        }
    };
    (int_range(min = $min:expr, max = $max:expr)) => {
        $crate::ValidationRule::IntRange {
            min: Some($min),
            max: Some($max),
            message: None,
        }
    };
    (allowed_values($values:expr)) => {
        $crate::ValidationRule::AllowedValues {
            values: $values.iter().map(|s| s.to_string()).collect(),
            message: None,
        }
    };
}
