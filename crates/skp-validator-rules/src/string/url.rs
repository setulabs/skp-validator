//! URL validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// URL validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::url::UrlRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = UrlRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("https://example.com", &ctx).is_ok());
/// assert!(rule.validate("not-a-url", &ctx).is_err());
/// ```
#[derive(Debug, Clone, Default)]
pub struct UrlRule {
    /// Allowed schemes (empty = all)
    pub schemes: Vec<String>,
    /// Custom error message
    pub message: Option<String>,
}

impl UrlRule {
    /// Create a new URL rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set allowed schemes (e.g., ["http", "https"]).
    pub fn schemes(mut self, schemes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.schemes = schemes.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "Must be a valid URL".to_string())
    }
}

impl Rule<str> for UrlRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        // Empty is valid (use required for non-empty)
        if value.is_empty() {
            return Ok(());
        }

        match url::Url::parse(value) {
            Ok(parsed_url) => {
                // Check scheme if restrictions are specified
                if !self.schemes.is_empty() {
                    let scheme = parsed_url.scheme();
                    if !self.schemes.iter().any(|s| s == scheme) {
                        return Err(ValidationErrors::from_iter([
                            ValidationError::new("", "url.scheme", format!(
                                "URL scheme '{}' is not allowed. Allowed: {:?}",
                                scheme, self.schemes
                            ))
                        ]));
                    }
                }
                Ok(())
            }
            Err(_) => {
                Err(ValidationErrors::from_iter([
                    ValidationError::new("", "url", self.get_message())
                ]))
            }
        }
    }

    fn name(&self) -> &'static str {
        "url"
    }

    fn default_message(&self) -> String {
        "Must be a valid URL".to_string()
    }
}

impl Rule<String> for UrlRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "url"
    }

    fn default_message(&self) -> String {
        "Must be a valid URL".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let rule = UrlRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("https://example.com", &ctx).is_ok());
        assert!(rule.validate("http://example.com/path?query=1", &ctx).is_ok());
        assert!(rule.validate("ftp://ftp.example.com", &ctx).is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        let rule = UrlRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("not-a-url", &ctx).is_err());
        assert!(rule.validate("example.com", &ctx).is_err()); // No scheme
    }

    #[test]
    fn test_scheme_restriction() {
        let rule = UrlRule::new().schemes(["https"]);
        let ctx = ValidationContext::default();

        assert!(rule.validate("https://example.com", &ctx).is_ok());
        assert!(rule.validate("http://example.com", &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = UrlRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }
}
