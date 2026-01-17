//! ASCII validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// ASCII validation rule - value must contain only ASCII characters.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::ascii::AsciiRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = AsciiRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("hello123", &ctx).is_ok());
/// assert!(rule.validate("héllo", &ctx).is_err()); // Contains non-ASCII
/// ```
#[derive(Debug, Clone, Default)]
pub struct AsciiRule {
    /// Allow only printable ASCII (32-126)
    pub printable_only: bool,
    /// Custom error message
    pub message: Option<String>,
}

impl AsciiRule {
    /// Create a new ASCII rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Only allow printable ASCII characters (32-126).
    pub fn printable(mut self) -> Self {
        self.printable_only = true;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            if self.printable_only {
                "Must contain only printable ASCII characters".to_string()
            } else {
                "Must contain only ASCII characters".to_string()
            }
        })
    }
}

impl Rule<str> for AsciiRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        let valid = if self.printable_only {
            value.chars().all(|c| c.is_ascii() && (' '..='~').contains(&c))
        } else {
            value.is_ascii()
        };

        if valid {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::root("ascii", self.get_message())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "ascii"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for AsciiRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "ascii"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii() {
        let rule = AsciiRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("hello123", &ctx).is_ok());
        assert!(rule.validate("Hello, World!", &ctx).is_ok());
        assert!(rule.validate("héllo", &ctx).is_err()); // é is non-ASCII
        assert!(rule.validate("你好", &ctx).is_err());
    }

    #[test]
    fn test_printable_only() {
        let rule = AsciiRule::new().printable();
        let ctx = ValidationContext::default();

        assert!(rule.validate("hello", &ctx).is_ok());
        assert!(rule.validate("hello\t", &ctx).is_err()); // Tab is not printable
        assert!(rule.validate("hello\n", &ctx).is_err()); // Newline is not printable
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = AsciiRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }
}
