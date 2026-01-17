//! Multiple of validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Multiple of validation rule - value must be divisible by the specified number.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::numeric::multiple_of::MultipleOfRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = MultipleOfRule::new(5.0);
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate(&10.0, &ctx).is_ok());
/// assert!(rule.validate(&7.0, &ctx).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct MultipleOfRule<T> {
    /// The divisor
    pub value: T,
    /// Custom error message
    pub message: Option<String>,
}

impl<T> MultipleOfRule<T> {
    /// Create a new multiple_of rule.
    pub fn new(value: T) -> Self {
        Self {
            value,
            message: None,
        }
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

impl MultipleOfRule<f64> {
    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Must be a multiple of {}", self.value)
        })
    }
}

impl MultipleOfRule<i64> {
    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Must be a multiple of {}", self.value)
        })
    }
}

impl Rule<f64> for MultipleOfRule<f64> {
    fn validate(&self, value: &f64, _ctx: &ValidationContext) -> ValidationResult<()> {
        if self.value == 0.0 {
            return Ok(()); // Division by zero check
        }

        let remainder = value % self.value;
        // Use epsilon for floating point comparison
        if remainder.abs() < f64::EPSILON || (self.value - remainder.abs()).abs() < f64::EPSILON {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::root("multiple_of", self.get_message())
                    .with_param("divisor", self.value)
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "multiple_of"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<f32> for MultipleOfRule<f64> {
    fn validate(&self, value: &f32, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<f64>>::validate(self, &(*value as f64), ctx)
    }

    fn name(&self) -> &'static str {
        "multiple_of"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<i64> for MultipleOfRule<i64> {
    fn validate(&self, value: &i64, _ctx: &ValidationContext) -> ValidationResult<()> {
        if self.value == 0 {
            return Ok(()); // Division by zero check
        }

        if value % self.value == 0 {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::root("multiple_of", self.get_message())
                    .with_param("divisor", self.value)
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "multiple_of"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<i32> for MultipleOfRule<i64> {
    fn validate(&self, value: &i32, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<i64>>::validate(self, &(*value as i64), ctx)
    }

    fn name(&self) -> &'static str {
        "multiple_of"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_of_float() {
        let rule = MultipleOfRule::new(5.0);
        let ctx = ValidationContext::default();

        assert!(rule.validate(&10.0, &ctx).is_ok());
        assert!(rule.validate(&15.0, &ctx).is_ok());
        assert!(rule.validate(&7.0, &ctx).is_err());
    }

    #[test]
    fn test_multiple_of_int() {
        let rule = MultipleOfRule::new(3_i64);
        let ctx = ValidationContext::default();

        assert!(rule.validate(&9_i64, &ctx).is_ok());
        assert!(rule.validate(&12_i64, &ctx).is_ok());
        assert!(rule.validate(&7_i64, &ctx).is_err());
    }

    #[test]
    fn test_custom_message() {
        let rule = MultipleOfRule::new(10.0).message("Must be a multiple of 10");
        assert_eq!(rule.get_message(), "Must be a multiple of 10");
    }
}
