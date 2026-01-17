//! Pattern (regex) validation rule.

use once_cell::sync::Lazy;
use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use std::collections::HashMap;
use std::sync::RwLock;

/// Cache for compiled regex patterns
static REGEX_CACHE: Lazy<RwLock<HashMap<String, regex::Regex>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// Pattern (regex) validation rule.
///
/// Compiles and caches regex patterns for performance.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::pattern::PatternRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = PatternRule::new(r"^\d{5}(-\d{4})?$"); // US ZIP code
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("12345", &ctx).is_ok());
/// assert!(rule.validate("12345-6789", &ctx).is_ok());
/// assert!(rule.validate("1234", &ctx).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct PatternRule {
    /// The regex pattern
    pub pattern: String,
    /// Custom error message
    pub message: Option<String>,
}

impl PatternRule {
    /// Create a new pattern rule.
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            message: None,
        }
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "Does not match the required format".to_string())
    }

    /// Get or compile the regex pattern
    fn get_regex(&self) -> Result<regex::Regex, String> {
        // Try to get from cache first (read lock)
        {
            let cache = REGEX_CACHE.read().unwrap();
            if let Some(re) = cache.get(&self.pattern) {
                return Ok(re.clone());
            }
        }

        // Compile and cache (write lock)
        let re = regex::Regex::new(&self.pattern).map_err(|e| e.to_string())?;
        
        {
            let mut cache = REGEX_CACHE.write().unwrap();
            cache.insert(self.pattern.clone(), re.clone());
        }

        Ok(re)
    }
}

impl Rule<str> for PatternRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        // Empty is valid (use required for non-empty)
        if value.is_empty() {
            return Ok(());
        }

        let regex = match self.get_regex() {
            Ok(re) => re,
            Err(e) => {
                return Err(ValidationErrors::from_iter([
                    ValidationError::new("", "pattern.invalid", format!("Invalid regex pattern: {}", e))
                ]));
            }
        };

        if regex.is_match(value) {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "pattern", self.get_message())
                    .with_param("pattern", self.pattern.clone())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "pattern"
    }

    fn default_message(&self) -> String {
        "Does not match the required format".to_string()
    }
}

impl Rule<String> for PatternRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "pattern"
    }

    fn default_message(&self) -> String {
        "Does not match the required format".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_code() {
        let rule = PatternRule::new(r"^\d{5}(-\d{4})?$");
        let ctx = ValidationContext::default();

        assert!(rule.validate("12345", &ctx).is_ok());
        assert!(rule.validate("12345-6789", &ctx).is_ok());
        assert!(rule.validate("1234", &ctx).is_err());
        assert!(rule.validate("123456", &ctx).is_err());
    }

    #[test]
    fn test_alphanumeric() {
        let rule = PatternRule::new(r"^[a-zA-Z0-9]+$");
        let ctx = ValidationContext::default();

        assert!(rule.validate("abc123", &ctx).is_ok());
        assert!(rule.validate("ABC", &ctx).is_ok());
        assert!(rule.validate("abc_123", &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = PatternRule::new(r"^\d+$");
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }

    #[test]
    fn test_custom_message() {
        let rule = PatternRule::new(r"^\d{5}$").message("Must be a 5-digit ZIP code");
        let ctx = ValidationContext::default();

        let result = rule.validate("123", &ctx);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.to_string().contains("Must be a 5-digit ZIP code"));
    }

    #[test]
    fn test_invalid_regex() {
        let rule = PatternRule::new(r"[invalid(");
        let ctx = ValidationContext::default();

        let result = rule.validate("test", &ctx);
        assert!(result.is_err());
    }
}
