//! Age validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use chrono::{NaiveDate, Utc, Datelike};

/// Age validation rule - validates age derived from date of birth.
///
/// # Example
///
/// ```rust,ignore
/// use skp_validator_rules::temporal::age::AgeRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = AgeRule::new("%Y-%m-%d").min(18).max(120);
/// let ctx = ValidationContext::default();
///
/// // Note: This test is date-dependent
/// assert!(rule.validate("1990-01-15", &ctx).is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct AgeRule {
    /// Date format string
    pub date_format: String,
    /// Minimum age (inclusive)
    pub min: Option<u32>,
    /// Maximum age (inclusive)
    pub max: Option<u32>,
    /// Custom error message
    pub message: Option<String>,
}

impl AgeRule {
    /// Create a new age rule.
    pub fn new(date_format: impl Into<String>) -> Self {
        Self {
            date_format: date_format.into(),
            min: None,
            max: None,
            message: None,
        }
    }

    /// Set minimum age.
    pub fn min(mut self, min: u32) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum age.
    pub fn max(mut self, max: u32) -> Self {
        self.max = Some(max);
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Calculate age from date of birth.
    fn calculate_age(&self, dob: NaiveDate) -> u32 {
        let today = Utc::now().date_naive();
        let mut age = today.year() - dob.year();
        
        // Check if birthday hasn't occurred yet this year
        if today.month() < dob.month() || 
           (today.month() == dob.month() && today.day() < dob.day()) {
            age -= 1;
        }
        
        age as u32
    }
}

impl Rule<str> for AgeRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        let dob = match NaiveDate::parse_from_str(value, &self.date_format) {
            Ok(d) => d,
            Err(_) => {
                return Err(ValidationErrors::from_iter([
                    ValidationError::new("", "date", 
                        format!("Invalid date format, expected '{}'", self.date_format))
                ]));
            }
        };

        let age = self.calculate_age(dob);

        if let Some(min) = self.min {
            if age < min {
                return Err(ValidationErrors::from_iter([
                    ValidationError::new("", "age.min", 
                        self.message.clone().unwrap_or_else(|| format!("Must be at least {} years old", min)))
                        .with_param("min_age", min as i64)
                        .with_param("actual_age", age as i64)
                ]));
            }
        }

        if let Some(max) = self.max {
            if age > max {
                return Err(ValidationErrors::from_iter([
                    ValidationError::new("", "age.max", 
                        self.message.clone().unwrap_or_else(|| format!("Must be at most {} years old", max)))
                        .with_param("max_age", max as i64)
                        .with_param("actual_age", age as i64)
                ]));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "age"
    }

    fn default_message(&self) -> String {
        "Invalid age".to_string()
    }
}

impl Rule<String> for AgeRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "age"
    }

    fn default_message(&self) -> String {
        "Invalid age".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_age_calculation() {
        let rule = AgeRule::new("%Y-%m-%d").min(18);
        
        // Calculate a date that's definitely 18+ years ago
        let today = Utc::now().date_naive();
        let dob = NaiveDate::from_ymd_opt(today.year() - 25, 1, 1).unwrap();
        let age = rule.calculate_age(dob);
        
        assert!(age >= 24 && age <= 25);
    }

    #[test]
    fn test_age_min() {
        let rule = AgeRule::new("%Y-%m-%d").min(18);
        let ctx = ValidationContext::default();
        
        // Too young (born last year)
        let today = Utc::now().date_naive();
        let young_dob = NaiveDate::from_ymd_opt(today.year() - 5, 1, 1).unwrap()
            .format("%Y-%m-%d").to_string();
        
        assert!(rule.validate(&young_dob, &ctx).is_err());
    }
}
