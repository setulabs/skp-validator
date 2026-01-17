//! Alphanumeric validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Alphanumeric validation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlphanumericMode {
    /// Allow letters and digits (default)
    #[default]
    LettersAndDigits,
    /// Allow only letters
    LettersOnly,
    /// Allow only digits
    DigitsOnly,
}

/// Alphanumeric validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::alphanumeric::AlphanumericRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = AlphanumericRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("abc123", &ctx).is_ok());
/// assert!(rule.validate("abc-123", &ctx).is_err()); // Contains dash
/// ```
#[derive(Debug, Clone, Default)]
pub struct AlphanumericRule {
    /// Validation mode
    pub mode: AlphanumericMode,
    /// Allow underscores
    pub allow_underscore: bool,
    /// Allow dashes
    pub allow_dash: bool,
    /// Allow spaces
    pub allow_space: bool,
    /// Custom error message
    pub message: Option<String>,
}

impl AlphanumericRule {
    /// Create a new alphanumeric rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Only allow letters (no digits).
    pub fn letters_only(mut self) -> Self {
        self.mode = AlphanumericMode::LettersOnly;
        self
    }

    /// Only allow digits (no letters).
    pub fn digits_only(mut self) -> Self {
        self.mode = AlphanumericMode::DigitsOnly;
        self
    }

    /// Allow underscores.
    pub fn allow_underscore(mut self) -> Self {
        self.allow_underscore = true;
        self
    }

    /// Allow dashes.
    pub fn allow_dash(mut self) -> Self {
        self.allow_dash = true;
        self
    }

    /// Allow spaces.
    pub fn allow_space(mut self) -> Self {
        self.allow_space = true;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            match self.mode {
                AlphanumericMode::LettersAndDigits => "Must contain only letters and digits".to_string(),
                AlphanumericMode::LettersOnly => "Must contain only letters".to_string(),
                AlphanumericMode::DigitsOnly => "Must contain only digits".to_string(),
            }
        })
    }

    fn is_allowed_char(&self, c: char) -> bool {
        let base_check = match self.mode {
            AlphanumericMode::LettersAndDigits => c.is_alphanumeric(),
            AlphanumericMode::LettersOnly => c.is_alphabetic(),
            AlphanumericMode::DigitsOnly => c.is_ascii_digit(),
        };

        if base_check {
            return true;
        }

        if self.allow_underscore && c == '_' {
            return true;
        }

        if self.allow_dash && c == '-' {
            return true;
        }

        if self.allow_space && c == ' ' {
            return true;
        }

        false
    }
}

impl Rule<str> for AlphanumericRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        if value.chars().all(|c| self.is_allowed_char(c)) {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::root("alphanumeric", self.get_message())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "alphanumeric"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for AlphanumericRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "alphanumeric"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alphanumeric() {
        let rule = AlphanumericRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("abc123", &ctx).is_ok());
        assert!(rule.validate("ABC", &ctx).is_ok());
        assert!(rule.validate("123", &ctx).is_ok());
        assert!(rule.validate("abc-123", &ctx).is_err());
        assert!(rule.validate("abc_123", &ctx).is_err());
    }

    #[test]
    fn test_letters_only() {
        let rule = AlphanumericRule::new().letters_only();
        let ctx = ValidationContext::default();

        assert!(rule.validate("abcXYZ", &ctx).is_ok());
        assert!(rule.validate("abc123", &ctx).is_err());
    }

    #[test]
    fn test_digits_only() {
        let rule = AlphanumericRule::new().digits_only();
        let ctx = ValidationContext::default();

        assert!(rule.validate("12345", &ctx).is_ok());
        assert!(rule.validate("abc123", &ctx).is_err());
    }

    #[test]
    fn test_with_underscore() {
        let rule = AlphanumericRule::new().allow_underscore();
        let ctx = ValidationContext::default();

        assert!(rule.validate("abc_123", &ctx).is_ok());
        assert!(rule.validate("abc-123", &ctx).is_err());
    }

    #[test]
    fn test_with_all_extras() {
        let rule = AlphanumericRule::new()
            .allow_underscore()
            .allow_dash()
            .allow_space();
        let ctx = ValidationContext::default();

        assert!(rule.validate("abc_123-def xyz", &ctx).is_ok());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = AlphanumericRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }
}
