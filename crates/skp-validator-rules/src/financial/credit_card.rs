//! Credit card validation rule using Luhn algorithm.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Credit card validation rule using Luhn algorithm.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::financial::credit_card::CreditCardRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = CreditCardRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("4532015112830366", &ctx).is_ok()); // Valid test card
/// assert!(rule.validate("1234567890123456", &ctx).is_err());
/// ```
#[derive(Debug, Clone, Default)]
pub struct CreditCardRule {
    /// Custom error message
    pub message: Option<String>,
}

impl CreditCardRule {
    /// Create a new credit_card rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "Must be a valid credit card number".to_string())
    }

    /// Validate using Luhn algorithm
    fn luhn_check(&self, number: &str) -> bool {
        // Remove spaces and dashes
        let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();

        // Must be between 13 and 19 digits
        if digits.len() < 13 || digits.len() > 19 {
            return false;
        }

        let mut sum = 0;
        let mut double = false;

        // Process from right to left
        for c in digits.chars().rev() {
            if let Some(digit) = c.to_digit(10) {
                let mut value = digit;
                if double {
                    value *= 2;
                    if value > 9 {
                        value -= 9;
                    }
                }
                sum += value;
                double = !double;
            } else {
                return false;
            }
        }

        sum % 10 == 0
    }
}

impl Rule<str> for CreditCardRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        if self.luhn_check(value) {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "credit_card", self.get_message())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "credit_card"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for CreditCardRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "credit_card"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cards() {
        let rule = CreditCardRule::new();
        let ctx = ValidationContext::default();

        // Valid test credit card numbers (Luhn-valid)
        assert!(rule.validate("4532015112830366", &ctx).is_ok()); // Visa
        assert!(rule.validate("5425233430109903", &ctx).is_ok()); // Mastercard
        assert!(rule.validate("378282246310005", &ctx).is_ok());  // Amex
        assert!(rule.validate("4111111111111111", &ctx).is_ok()); // Test Visa
    }

    #[test]
    fn test_invalid_cards() {
        let rule = CreditCardRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("1234567890123456", &ctx).is_err());
        assert!(rule.validate("4111111111111112", &ctx).is_err()); // Changed last digit
    }

    #[test]
    fn test_with_spaces() {
        let rule = CreditCardRule::new();
        let ctx = ValidationContext::default();

        // Should handle spaces
        assert!(rule.validate("4532 0151 1283 0366", &ctx).is_ok());
    }

    #[test]
    fn test_with_dashes() {
        let rule = CreditCardRule::new();
        let ctx = ValidationContext::default();

        // Should handle dashes
        assert!(rule.validate("4532-0151-1283-0366", &ctx).is_ok());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = CreditCardRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }

    #[test]
    fn test_too_short() {
        let rule = CreditCardRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("123456789012", &ctx).is_err()); // 12 digits
    }
}
