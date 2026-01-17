//! Email validation rule.

use once_cell::sync::Lazy;
use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// HTML5 email regex (simpler, widely compatible)
static HTML5_EMAIL_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

/// Email validation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmailMode {
    /// HTML5 specification (default, simpler, widely compatible)
    #[default]
    Html5,
}

/// Email format validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::email::EmailRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = EmailRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("test@example.com", &ctx).is_ok());
/// assert!(rule.validate("invalid", &ctx).is_err());
/// ```
#[derive(Debug, Clone, Default)]
pub struct EmailRule {
    /// Validation mode
    pub mode: EmailMode,
    /// Custom error message
    pub message: Option<String>,
}

impl EmailRule {
    /// Create a new email rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set validation mode.
    pub fn mode(mut self, mode: EmailMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "Must be a valid email address".to_string())
    }
}

impl Rule<str> for EmailRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        // Empty is valid for email (use required for non-empty)
        if value.is_empty() {
            return Ok(());
        }

        let is_valid = HTML5_EMAIL_REGEX.is_match(value);

        if is_valid {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::root("email", self.get_message())
                    .with_param("value", value.to_string())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "email"
    }

    fn default_message(&self) -> String {
        "Must be a valid email address".to_string()
    }
}

impl Rule<String> for EmailRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "email"
    }

    fn default_message(&self) -> String {
        "Must be a valid email address".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        let rule = EmailRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("test@example.com", &ctx).is_ok());
        assert!(rule.validate("user.name+tag@example.co.uk", &ctx).is_ok());
        assert!(rule.validate("valid@subdomain.example.org", &ctx).is_ok());
        assert!(rule.validate("user123@test.io", &ctx).is_ok());
    }

    #[test]
    fn test_invalid_emails() {
        let rule = EmailRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("invalid", &ctx).is_err());
        assert!(rule.validate("@example.com", &ctx).is_err());
        assert!(rule.validate("test@", &ctx).is_err());
        assert!(rule.validate("test@.com", &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = EmailRule::new();
        let ctx = ValidationContext::default();

        // Empty is valid - use required rule for non-empty
        assert!(rule.validate("", &ctx).is_ok());
    }

    #[test]
    fn test_custom_message() {
        let rule = EmailRule::new().message("Please enter a valid email");
        let ctx = ValidationContext::default();

        let result = rule.validate("invalid", &ctx);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.to_string().contains("Please enter a valid email"));
    }
}
