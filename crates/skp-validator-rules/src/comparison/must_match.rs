//! Must match validation rule for field comparison.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Must match validation rule - field must equal another field.
///
/// This is commonly used for password confirmation.
///
/// # Example
///
/// ```rust,ignore
/// use skp_validator_rules::comparison::must_match::MustMatchRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// // In a struct context, you would compare password_confirm to password
/// let rule = MustMatchRule::new("password");
/// ```
#[derive(Debug, Clone)]
pub struct MustMatchRule {
    /// The field to match against
    pub other_field: String,
    /// Custom error message
    pub message: Option<String>,
}

impl MustMatchRule {
    /// Create a new must_match rule.
    pub fn new(other_field: impl Into<String>) -> Self {
        Self {
            other_field: other_field.into(),
            message: None,
        }
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Must match '{}'", self.other_field)
        })
    }
}

impl Rule<str> for MustMatchRule {
    fn validate(&self, value: &str, ctx: &ValidationContext) -> ValidationResult<()> {
        // Get the other field's value from context
        if let Some(other_value) = ctx.get_string(&self.other_field) {
            if value == other_value {
                return Ok(());
            } else {
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("must_match", self.get_message())
                        .with_param("other_field", self.other_field.clone())
                ]));
            }
        }

        // If other field not found, validation passes (the other field's validation will fail)
        Ok(())
    }

    fn name(&self) -> &'static str {
        "must_match"
    }

    fn default_message(&self) -> String {
        format!("Must match '{}'", self.other_field)
    }
}

impl Rule<String> for MustMatchRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "must_match"
    }

    fn default_message(&self) -> String {
        format!("Must match '{}'", self.other_field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn test_must_match() {
        use serde_json::json;
        
        let rule = MustMatchRule::new("password");
        let ctx = ValidationContext::from_json(&json!({
            "password": "secret123",
            "password_confirm": "secret123"
        }));

        // Matches
        assert!(rule.validate("secret123", &ctx).is_ok());
        
        // Doesn't match
        assert!(rule.validate("different", &ctx).is_err());
    }

    #[test]
    fn test_custom_message() {
        let rule = MustMatchRule::new("password").message("Passwords must match");
        assert_eq!(rule.get_message(), "Passwords must match");
    }
}
