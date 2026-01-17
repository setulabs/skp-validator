//! Date validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use chrono::NaiveDate;

/// Date validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::temporal::date::DateRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = DateRule::new("%Y-%m-%d");
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("2024-01-15", &ctx).is_ok());
/// assert!(rule.validate("invalid", &ctx).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct DateRule {
    /// Date format string (strftime format)
    pub format: String,
    /// Custom error message
    pub message: Option<String>,
}

impl DateRule {
    /// Create a new date rule with the specified format.
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            message: None,
        }
    }

    /// Create with ISO 8601 format (YYYY-MM-DD).
    pub fn iso8601() -> Self {
        Self::new("%Y-%m-%d")
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Must be a valid date in format '{}'", self.format)
        })
    }
}

impl Rule<str> for DateRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        // Empty is valid (use required for non-empty)
        if value.is_empty() {
            return Ok(());
        }

        match NaiveDate::parse_from_str(value, &self.format) {
            Ok(_) => Ok(()),
            Err(_) => {
                Err(ValidationErrors::from_iter([
                    ValidationError::root("date", self.get_message())
                        .with_param("format", self.format.clone())
                ]))
            }
        }
    }

    fn name(&self) -> &'static str {
        "date"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for DateRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "date"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_dates() {
        let rule = DateRule::iso8601();
        let ctx = ValidationContext::default();

        assert!(rule.validate("2024-01-15", &ctx).is_ok());
        assert!(rule.validate("2000-12-31", &ctx).is_ok());
    }

    #[test]
    fn test_invalid_dates() {
        let rule = DateRule::iso8601();
        let ctx = ValidationContext::default();

        assert!(rule.validate("invalid", &ctx).is_err());
        assert!(rule.validate("2024-13-01", &ctx).is_err()); // Invalid month
        assert!(rule.validate("2024-02-30", &ctx).is_err()); // Invalid day
    }

    #[test]
    fn test_custom_format() {
        let rule = DateRule::new("%d/%m/%Y");
        let ctx = ValidationContext::default();

        assert!(rule.validate("15/01/2024", &ctx).is_ok());
        assert!(rule.validate("2024-01-15", &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = DateRule::iso8601();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }
}
