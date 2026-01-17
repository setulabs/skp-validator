//! Contains, prefix, and suffix validation rules.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Contains validation rule - string must contain a substring.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::contains::ContainsRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = ContainsRule::new("@");
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("test@example.com", &ctx).is_ok());
/// assert!(rule.validate("testexample.com", &ctx).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct ContainsRule {
    /// The substring to search for
    pub substring: String,
    /// Case insensitive matching
    pub case_insensitive: bool,
    /// Custom error message
    pub message: Option<String>,
}

impl ContainsRule {
    /// Create a new contains rule.
    pub fn new(substring: impl Into<String>) -> Self {
        Self {
            substring: substring.into(),
            case_insensitive: false,
            message: None,
        }
    }

    /// Enable case-insensitive matching.
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Must contain '{}'", self.substring)
        })
    }
}

impl Rule<str> for ContainsRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        let contains = if self.case_insensitive {
            value.to_lowercase().contains(&self.substring.to_lowercase())
        } else {
            value.contains(&self.substring)
        };

        if contains {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "contains", self.get_message())
                    .with_param("substring", self.substring.clone())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "contains"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for ContainsRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "contains"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

/// Prefix validation rule - string must start with a prefix.
#[derive(Debug, Clone)]
pub struct PrefixRule {
    /// The prefix to match
    pub prefix: String,
    /// Case insensitive matching
    pub case_insensitive: bool,
    /// Custom error message
    pub message: Option<String>,
}

impl PrefixRule {
    /// Create a new prefix rule.
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            case_insensitive: false,
            message: None,
        }
    }

    /// Enable case-insensitive matching.
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Must start with '{}'", self.prefix)
        })
    }
}

impl Rule<str> for PrefixRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        let starts_with = if self.case_insensitive {
            value.to_lowercase().starts_with(&self.prefix.to_lowercase())
        } else {
            value.starts_with(&self.prefix)
        };

        if starts_with {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "prefix", self.get_message())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "prefix"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for PrefixRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "prefix"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

/// Suffix validation rule - string must end with a suffix.
#[derive(Debug, Clone)]
pub struct SuffixRule {
    /// The suffix to match
    pub suffix: String,
    /// Case insensitive matching
    pub case_insensitive: bool,
    /// Custom error message
    pub message: Option<String>,
}

impl SuffixRule {
    /// Create a new suffix rule.
    pub fn new(suffix: impl Into<String>) -> Self {
        Self {
            suffix: suffix.into(),
            case_insensitive: false,
            message: None,
        }
    }

    /// Enable case-insensitive matching.
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Must end with '{}'", self.suffix)
        })
    }
}

impl Rule<str> for SuffixRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        let ends_with = if self.case_insensitive {
            value.to_lowercase().ends_with(&self.suffix.to_lowercase())
        } else {
            value.ends_with(&self.suffix)
        };

        if ends_with {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "suffix", self.get_message())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "suffix"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for SuffixRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "suffix"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let rule = ContainsRule::new("@");
        let ctx = ValidationContext::default();

        assert!(rule.validate("test@example.com", &ctx).is_ok());
        assert!(rule.validate("testexample.com", &ctx).is_err());
    }

    #[test]
    fn test_contains_case_insensitive() {
        let rule = ContainsRule::new("HELLO").case_insensitive();
        let ctx = ValidationContext::default();

        assert!(rule.validate("say hello world", &ctx).is_ok());
    }

    #[test]
    fn test_prefix() {
        let rule = PrefixRule::new("https://");
        let ctx = ValidationContext::default();

        assert!(rule.validate("https://example.com", &ctx).is_ok());
        assert!(rule.validate("http://example.com", &ctx).is_err());
    }

    #[test]
    fn test_suffix() {
        let rule = SuffixRule::new(".pdf");
        let ctx = ValidationContext::default();

        assert!(rule.validate("document.pdf", &ctx).is_ok());
        assert!(rule.validate("document.doc", &ctx).is_err());
    }

    #[test]
    fn test_suffix_case_insensitive() {
        let rule = SuffixRule::new(".pdf").case_insensitive();
        let ctx = ValidationContext::default();

        assert!(rule.validate("document.PDF", &ctx).is_ok());
    }

    #[test]
    fn test_empty_is_valid() {
        let contains = ContainsRule::new("test");
        let prefix = PrefixRule::new("test");
        let suffix = SuffixRule::new("test");
        let ctx = ValidationContext::default();

        assert!(contains.validate("", &ctx).is_ok());
        assert!(prefix.validate("", &ctx).is_ok());
        assert!(suffix.validate("", &ctx).is_ok());
    }
}
