//! UUID validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use uuid::Uuid;

/// UUID validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::uuid_rule::UuidRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = UuidRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("550e8400-e29b-41d4-a716-446655440000", &ctx).is_ok());
/// assert!(rule.validate("invalid", &ctx).is_err());
/// ```
#[derive(Debug, Clone, Default)]
pub struct UuidRule {
    /// Require specific version (1-5)
    pub version: Option<u8>,
    /// Custom error message
    pub message: Option<String>,
}

impl UuidRule {
    /// Create a new UUID rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require a specific UUID version.
    pub fn version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "Must be a valid UUID".to_string())
    }
}

impl Rule<str> for UuidRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(());
        }

        match Uuid::parse_str(value) {
            Ok(uuid) => {
                // Check version if specified
                if let Some(expected_version) = self.version {
                    let actual_version = uuid.get_version_num();
                    if actual_version != expected_version as usize {
                        return Err(ValidationErrors::from_iter([
                            ValidationError::new("", "uuid.version", 
                                format!("Expected UUID version {}, got {}", expected_version, actual_version))
                        ]));
                    }
                }
                Ok(())
            }
            Err(_) => {
                Err(ValidationErrors::from_iter([
                    ValidationError::new("", "uuid", self.get_message())
                ]))
            }
        }
    }

    fn name(&self) -> &'static str {
        "uuid"
    }

    fn default_message(&self) -> String {
        "Must be a valid UUID".to_string()
    }
}

impl Rule<String> for UuidRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "uuid"
    }

    fn default_message(&self) -> String {
        "Must be a valid UUID".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_uuid() {
        let rule = UuidRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("550e8400-e29b-41d4-a716-446655440000", &ctx).is_ok());
        assert!(rule.validate("6ba7b810-9dad-11d1-80b4-00c04fd430c8", &ctx).is_ok());
    }

    #[test]
    fn test_invalid_uuid() {
        let rule = UuidRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("invalid", &ctx).is_err());
        assert!(rule.validate("123-456-789", &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = UuidRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }
}
