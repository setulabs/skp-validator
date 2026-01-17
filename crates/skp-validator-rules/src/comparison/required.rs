//! Required validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Required validation rule - field must not be empty/null.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::comparison::required::RequiredRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = RequiredRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("hello", &ctx).is_ok());
/// assert!(rule.validate("", &ctx).is_err()); // Empty string
/// ```
#[derive(Debug, Clone, Default)]
pub struct RequiredRule {
    /// Custom error message
    pub message: Option<String>,
}

impl RequiredRule {
    /// Create a new required rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
    
    /// Get the error message (custom or default).
    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "This field is required".to_string())
    }
}

impl Rule<str> for RequiredRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.trim().is_empty() {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "required", self.get_message())
            ]))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }

    fn default_message(&self) -> String {
        "This field is required".to_string()
    }
}

impl Rule<String> for RequiredRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "required"
    }

    fn default_message(&self) -> String {
        "This field is required".to_string()
    }
}

impl<T> Rule<Option<T>> for RequiredRule {
    fn validate(&self, value: &Option<T>, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_none() {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "required", self.get_message())
            ]))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }

    fn default_message(&self) -> String {
        "This field is required".to_string()
    }
}

impl<T> Rule<Vec<T>> for RequiredRule {
    fn validate(&self, value: &Vec<T>, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "required", self.get_message())
            ]))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }

    fn default_message(&self) -> String {
        "This field is required".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_string() {
        let rule = RequiredRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("hello", &ctx).is_ok());
        assert!(rule.validate("", &ctx).is_err());
        assert!(rule.validate("   ", &ctx).is_err()); // Whitespace only
    }

    #[test]
    fn test_required_option() {
        let rule = RequiredRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate(&Some(42), &ctx).is_ok());
        assert!(rule.validate(&None::<i32>, &ctx).is_err());
    }

    #[test]
    fn test_required_vec() {
        let rule = RequiredRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate(&vec![1, 2, 3], &ctx).is_ok());
        assert!(rule.validate(&Vec::<i32>::new(), &ctx).is_err());
    }

    #[test]
    fn test_custom_message() {
        let rule = RequiredRule::new().message("Name cannot be empty");
        let ctx = ValidationContext::default();

        let result = rule.validate("", &ctx);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.to_string().contains("Name cannot be empty"));
    }
}
