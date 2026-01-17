//! Allowed values validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use std::fmt::Display;

/// Allowed values validation rule - field must be one of the specified values.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::comparison::allowed_values::AllowedValuesRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = AllowedValuesRule::new(vec!["active", "pending", "disabled"]);
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("active", &ctx).is_ok());
/// assert!(rule.validate("unknown", &ctx).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct AllowedValuesRule<T> {
    /// The allowed values
    pub values: Vec<T>,
    /// Custom error message
    pub message: Option<String>,
}

impl<T> AllowedValuesRule<T> {
    /// Create a new allowed values rule.
    pub fn new(values: Vec<T>) -> Self {
        Self {
            values,
            message: None,
        }
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

impl<T: Display> AllowedValuesRule<T> {
    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            let values_str: Vec<String> = self.values.iter().map(|v| v.to_string()).collect();
            format!("Must be one of: {}", values_str.join(", "))
        })
    }
}

impl AllowedValuesRule<String> {
    /// Create from string slices.
    pub fn from_strs(values: &[&str]) -> Self {
        Self {
            values: values.iter().map(|s| s.to_string()).collect(),
            message: None,
        }
    }
}

impl Rule<str> for AllowedValuesRule<String> {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        // Empty is valid (use required for non-empty)
        if value.is_empty() {
            return Ok(());
        }

        if self.values.iter().any(|v| v == value) {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "allowed_values", self.get_message())
                    .with_param("allowed", self.values.clone())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "allowed_values"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for AllowedValuesRule<String> {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "allowed_values"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

// Implement for &str
impl Rule<str> for AllowedValuesRule<&str> {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        if self.values.contains(&value) {
            Ok(())
        } else {
            let values_str: Vec<String> = self.values.iter().map(|v| v.to_string()).collect();
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "allowed_values", 
                    self.message.clone().unwrap_or_else(|| format!("Must be one of: {}", values_str.join(", "))))
                    .with_param("allowed", values_str)
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "allowed_values"
    }

    fn default_message(&self) -> String {
        let values_str: Vec<String> = self.values.iter().map(|v| v.to_string()).collect();
        format!("Must be one of: {}", values_str.join(", "))
    }
}

// Implement for numeric types
macro_rules! impl_allowed_values_numeric {
    ($($t:ty),+) => {
        $(
            impl Rule<$t> for AllowedValuesRule<$t> {
                fn validate(&self, value: &$t, _ctx: &ValidationContext) -> ValidationResult<()> {
                    if self.values.contains(value) {
                        Ok(())
                    } else {
                        Err(ValidationErrors::from_iter([
                            ValidationError::new("", "allowed_values", self.get_message())
                        ]))
                    }
                }

                fn name(&self) -> &'static str {
                    "allowed_values"
                }

                fn default_message(&self) -> String {
                    self.get_message()
                }
            }
        )+
    };
}

impl_allowed_values_numeric!(i32, i64, u32, u64, f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_strings() {
        let rule = AllowedValuesRule::from_strs(&["active", "pending", "disabled"]);
        let ctx = ValidationContext::default();

        assert!(rule.validate("active", &ctx).is_ok());
        assert!(rule.validate("pending", &ctx).is_ok());
        assert!(rule.validate("unknown", &ctx).is_err());
    }

    #[test]
    fn test_allowed_numbers() {
        let rule = AllowedValuesRule::new(vec![1, 2, 3, 5, 8, 13]);
        let ctx = ValidationContext::default();

        assert!(rule.validate(&5, &ctx).is_ok());
        assert!(rule.validate(&4, &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = AllowedValuesRule::from_strs(&["a", "b"]);
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }

    #[test]
    fn test_custom_message() {
        let rule = AllowedValuesRule::from_strs(&["a", "b"]).message("Invalid status");
        let ctx = ValidationContext::default();

        let result = rule.validate("c", &ctx);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.to_string().contains("Invalid status"));
    }
}
