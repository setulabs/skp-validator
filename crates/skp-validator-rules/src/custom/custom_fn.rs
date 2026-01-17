//! Custom function validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use std::sync::Arc;

/// Custom function validation rule - validates using a user-provided function.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::custom::custom_fn::CustomFnRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = CustomFnRule::new("is_even", |value: &i32, _ctx| {
///     value % 2 == 0
/// }).message("Value must be even");
///
/// let ctx = ValidationContext::default();
/// assert!(rule.validate(&4, &ctx).is_ok());
/// assert!(rule.validate(&3, &ctx).is_err());
/// ```
pub struct CustomFnRule<T: ?Sized, F>
where
    F: Fn(&T, &ValidationContext) -> bool + Send + Sync,
{
    /// Rule name for error reporting
    pub rule_name: String,
    /// The validation function
    pub validator: Arc<F>,
    /// Custom error message
    pub message: Option<String>,
    /// Phantom data for T
    _marker: std::marker::PhantomData<fn(&T)>,
}

impl<T: ?Sized, F> CustomFnRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> bool + Send + Sync,
{
    /// Create a new custom function rule.
    pub fn new(name: impl Into<String>, validator: F) -> Self {
        Self {
            rule_name: name.into(),
            validator: Arc::new(validator),
            message: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            format!("Validation failed for rule '{}'", self.rule_name)
        })
    }
}

impl<T: ?Sized, F> std::fmt::Debug for CustomFnRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> bool + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomFnRule")
            .field("rule_name", &self.rule_name)
            .field("message", &self.message)
            .finish()
    }
}

impl<T: ?Sized, F> Clone for CustomFnRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> bool + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            rule_name: self.rule_name.clone(),
            validator: Arc::clone(&self.validator),
            message: self.message.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized, F> Rule<T> for CustomFnRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> bool + Send + Sync,
{
    fn validate(&self, value: &T, ctx: &ValidationContext) -> ValidationResult<()> {
        if (self.validator)(value, ctx) {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::root(&self.rule_name, self.get_message())
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "custom"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

/// Custom validation with result - allows returning detailed errors.
pub struct CustomResultRule<T: ?Sized, F>
where
    F: Fn(&T, &ValidationContext) -> ValidationResult<()> + Send + Sync,
{
    /// Rule name for error reporting
    pub rule_name: String,
    /// The validation function
    pub validator: Arc<F>,
    /// Phantom data for T
    _marker: std::marker::PhantomData<fn(&T)>,
}

impl<T: ?Sized, F> CustomResultRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> ValidationResult<()> + Send + Sync,
{
    /// Create a new custom result rule.
    pub fn new(name: impl Into<String>, validator: F) -> Self {
        Self {
            rule_name: name.into(),
            validator: Arc::new(validator),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized, F> std::fmt::Debug for CustomResultRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> ValidationResult<()> + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomResultRule")
            .field("rule_name", &self.rule_name)
            .finish()
    }
}

impl<T: ?Sized, F> Clone for CustomResultRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> ValidationResult<()> + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            rule_name: self.rule_name.clone(),
            validator: Arc::clone(&self.validator),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized, F> Rule<T> for CustomResultRule<T, F>
where
    F: Fn(&T, &ValidationContext) -> ValidationResult<()> + Send + Sync,
{
    fn validate(&self, value: &T, ctx: &ValidationContext) -> ValidationResult<()> {
        (self.validator)(value, ctx)
    }

    fn name(&self) -> &'static str {
        "custom"
    }

    fn default_message(&self) -> String {
        format!("Validation failed for rule '{}'", self.rule_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_fn() {
        let rule = CustomFnRule::new("is_positive", |value: &i32, _ctx: &ValidationContext| {
            *value > 0
        });
        let ctx = ValidationContext::default();

        assert!(rule.validate(&5, &ctx).is_ok());
        assert!(rule.validate(&-1, &ctx).is_err());
    }

    #[test]
    fn test_custom_message() {
        let rule = CustomFnRule::new("is_even", |value: &i32, _ctx: &ValidationContext| {
            value % 2 == 0
        }).message("Must be an even number");
        let ctx = ValidationContext::default();

        let result = rule.validate(&3, &ctx);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.to_string().contains("Must be an even number"));
    }

    #[test]
    fn test_custom_result() {
        let rule = CustomResultRule::new("complex_validation", |value: &i32, _ctx: &ValidationContext| {
            if *value < 0 {
                Err(ValidationErrors::from_iter([
                    ValidationError::root("positive", "Must be positive")
                ]))
            } else if *value > 100 {
                Err(ValidationErrors::from_iter([
                    ValidationError::root("max", "Must be <= 100")
                ]))
            } else {
                Ok(())
            }
        });
        let ctx = ValidationContext::default();

        assert!(rule.validate(&50, &ctx).is_ok());
        assert!(rule.validate(&-1, &ctx).is_err());
        assert!(rule.validate(&101, &ctx).is_err());
    }
}
