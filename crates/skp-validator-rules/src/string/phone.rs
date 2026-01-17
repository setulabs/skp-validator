//! Phone number validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Phone number validation rule.
///
/// Uses the libphonenumber library for validation.
///
/// # Example
///
/// ```rust,ignore
/// use skp_validator_rules::string::phone::PhoneRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = PhoneRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("+1234567890", &ctx).is_ok());
/// ```
#[derive(Debug, Clone, Default)]
pub struct PhoneRule {
    /// Default country code (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// Custom error message
    pub message: Option<String>,
}

impl PhoneRule {
    /// Create a new phone rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default country code.
    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "Must be a valid phone number".to_string())
    }
}

impl Rule<str> for PhoneRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        // Try to parse with country hint
        let result = if let Some(ref country) = self.country {
            phonenumber::parse(Some(country.parse().unwrap_or(phonenumber::country::Id::US)), value)
        } else {
            // Try without country hint
            phonenumber::parse(None, value)
        };

        match result {
            Ok(phone) => {
                if phonenumber::is_valid(&phone) {
                    Ok(())
                } else {
                    Err(ValidationErrors::from_iter([
                        ValidationError::new("", "phone.invalid", self.get_message())
                    ]))
                }
            }
            Err(_) => {
                Err(ValidationErrors::from_iter([
                    ValidationError::new("", "phone", self.get_message())
                ]))
            }
        }
    }

    fn name(&self) -> &'static str {
        "phone"
    }

    fn default_message(&self) -> String {
        "Must be a valid phone number".to_string()
    }
}

impl Rule<String> for PhoneRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "phone"
    }

    fn default_message(&self) -> String {
        "Must be a valid phone number".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_phone_with_country_code() {
        let rule = PhoneRule::new();
        let ctx = ValidationContext::default();

        // International format
        assert!(rule.validate("+14155552671", &ctx).is_ok());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = PhoneRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }
}
