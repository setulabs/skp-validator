//! Contextual validation rule - validates based on context metadata.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use std::sync::Arc;

/// Contextual validation rule - validates based on context metadata or locale.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::custom::contextual::ContextualRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// // A rule that only validates when context has "strict" mode
/// let rule = ContextualRule::new("strict_email")
///     .when(|ctx| ctx.get_meta("mode") == Some("strict"))
///     .validate_with(|value: &str, _ctx| {
///         // More strict email validation
///         value.contains('@') && value.contains('.') && value.len() > 5
///     });
///
/// let strict_ctx = ValidationContext::new().with_meta("mode", "strict");
/// let normal_ctx = ValidationContext::new();
///
/// // In strict mode, validation is applied
/// assert!(rule.validate("a@b.c", &strict_ctx).is_err()); // Too short
///
/// // In normal mode, validation is skipped
/// assert!(rule.validate("a@b.c", &normal_ctx).is_ok());
/// ```
pub struct ContextualRule<T>
where
    T: ?Sized,
{
    /// Rule name
    pub rule_name: String,
    /// Condition function - when to apply validation
    pub condition: Option<Arc<dyn Fn(&ValidationContext) -> bool + Send + Sync>>,
    /// Validation function
    pub validator: Option<Arc<dyn Fn(&T, &ValidationContext) -> bool + Send + Sync>>,
    /// Custom error message
    pub message: Option<String>,
}

impl<T: ?Sized> ContextualRule<T> {
    /// Create a new contextual rule.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            rule_name: name.into(),
            condition: None,
            validator: None,
            message: None,
        }
    }

    /// Set condition for when to apply validation.
    pub fn when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&ValidationContext) -> bool + Send + Sync + 'static,
    {
        self.condition = Some(Arc::new(condition));
        self
    }

    /// Set the validation function.
    pub fn validate_with<F>(mut self, validator: F) -> Self
    where
        F: Fn(&T, &ValidationContext) -> bool + Send + Sync + 'static,
    {
        self.validator = Some(Arc::new(validator));
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Contextual validation '{}' failed", self.rule_name)
        })
    }
}

impl<T: ?Sized> std::fmt::Debug for ContextualRule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextualRule")
            .field("rule_name", &self.rule_name)
            .field("has_condition", &self.condition.is_some())
            .field("has_validator", &self.validator.is_some())
            .finish()
    }
}

impl<T: ?Sized + 'static> Rule<T> for ContextualRule<T> {
    fn validate(&self, value: &T, ctx: &ValidationContext) -> ValidationResult<()> {
        // Check if condition is met
        if let Some(ref condition) = self.condition {
            if !condition(ctx) {
                // Condition not met, skip validation
                return Ok(());
            }
        }

        // Run validation if validator is set
        if let Some(ref validator) = self.validator {
            if validator(value, ctx) {
                Ok(())
            } else {
                Err(ValidationErrors::from_iter([
                    ValidationError::root(&self.rule_name, self.get_message())
                ]))
            }
        } else {
            // No validator set, pass
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "contextual"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contextual_applies() {
        let rule = ContextualRule::<str>::new("strict")
            .when(|ctx| ctx.get_meta("mode") == Some("strict"))
            .validate_with(|value: &str, _ctx| value.len() >= 5);

        let strict_ctx = ValidationContext::new().with_meta("mode", "strict");
        
        assert!(rule.validate("hello", &strict_ctx).is_ok());
        assert!(rule.validate("hi", &strict_ctx).is_err());
    }

    #[test]
    fn test_contextual_skips() {
        let rule = ContextualRule::<str>::new("strict")
            .when(|ctx| ctx.get_meta("mode") == Some("strict"))
            .validate_with(|value: &str, _ctx| value.len() >= 5);

        let normal_ctx = ValidationContext::new();
        
        // Validation is skipped when condition is not met
        assert!(rule.validate("hi", &normal_ctx).is_ok());
    }
}
