//! Date range validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use chrono::NaiveDate;

/// Date range validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::temporal::date_range::DateRangeRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = DateRangeRule::new("%Y-%m-%d")
///     .min("2020-01-01")
///     .max("2030-12-31");
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("2024-01-15", &ctx).is_ok());
/// assert!(rule.validate("2019-01-01", &ctx).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct DateRangeRule {
    /// Date format string
    pub format: String,
    /// Minimum date (inclusive)
    pub min: Option<String>,
    /// Maximum date (inclusive)
    pub max: Option<String>,
    /// Custom error message
    pub message: Option<String>,
}

impl DateRangeRule {
    /// Create a new date range rule.
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            min: None,
            max: None,
            message: None,
        }
    }

    /// Set minimum date.
    pub fn min(mut self, min: impl Into<String>) -> Self {
        self.min = Some(min.into());
        self
    }

    /// Set maximum date.
    pub fn max(mut self, max: impl Into<String>) -> Self {
        self.max = Some(max.into());
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

impl Rule<str> for DateRangeRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        let date = match NaiveDate::parse_from_str(value, &self.format) {
            Ok(d) => d,
            Err(_) => {
                return Err(ValidationErrors::from_iter([
                    ValidationError::new("", "date", format!("Invalid date format, expected '{}'", self.format))
                ]));
            }
        };

        if let Some(ref min_str) = self.min {
            if let Ok(min_date) = NaiveDate::parse_from_str(min_str, &self.format) {
                if date < min_date {
                    return Err(ValidationErrors::from_iter([
                        ValidationError::new("", "date_range.min", 
                            self.message.clone().unwrap_or_else(|| format!("Date must be on or after {}", min_str)))
                    ]));
                }
            }
        }

        if let Some(ref max_str) = self.max {
            if let Ok(max_date) = NaiveDate::parse_from_str(max_str, &self.format) {
                if date > max_date {
                    return Err(ValidationErrors::from_iter([
                        ValidationError::new("", "date_range.max", 
                            self.message.clone().unwrap_or_else(|| format!("Date must be on or before {}", max_str)))
                    ]));
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "date_range"
    }

    fn default_message(&self) -> String {
        "Date out of range".to_string()
    }
}

impl Rule<String> for DateRangeRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "date_range"
    }

    fn default_message(&self) -> String {
        "Date out of range".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_range() {
        let rule = DateRangeRule::new("%Y-%m-%d")
            .min("2020-01-01")
            .max("2030-12-31");
        let ctx = ValidationContext::default();

        assert!(rule.validate("2024-01-15", &ctx).is_ok());
        assert!(rule.validate("2020-01-01", &ctx).is_ok()); // Boundary
        assert!(rule.validate("2030-12-31", &ctx).is_ok()); // Boundary
    }

    #[test]
    fn test_out_of_range() {
        let rule = DateRangeRule::new("%Y-%m-%d")
            .min("2020-01-01")
            .max("2030-12-31");
        let ctx = ValidationContext::default();

        assert!(rule.validate("2019-12-31", &ctx).is_err());
        assert!(rule.validate("2031-01-01", &ctx).is_err());
    }
}
