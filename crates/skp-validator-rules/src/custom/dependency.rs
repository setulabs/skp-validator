//! Field dependency validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use std::sync::Arc;

/// Dependency condition type
#[derive(Debug, Clone)]
pub enum DependencyCondition {
    /// Field must be present (not empty/null)
    Present,
    /// Field must be absent (empty/null)
    Absent,
    /// Field must equal a specific value
    Equals(String),
    /// Field must not equal a specific value
    NotEquals(String),
    /// Field must be one of these values
    In(Vec<String>),
    /// Field must not be one of these values
    NotIn(Vec<String>),
}

/// Field dependency validation rule - validates based on other field values.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::custom::dependency::{DependencyRule, DependencyCondition};
/// use skp_validator_core::{Rule, ValidationContext};
///
/// // Shipping address is required when shipping_method is "delivery"
/// let rule = DependencyRule::<str>::new("shipping_address")
///     .depends_on("shipping_method", DependencyCondition::Equals("delivery".to_string()))
///     .then_required();
/// ```
pub struct DependencyRule<T>
where
    T: ?Sized,
{
    /// The field being validated
    pub field_name: String,
    /// The field we depend on
    pub depends_on_field: Option<String>,
    /// The condition on the dependency field
    pub condition: Option<DependencyCondition>,
    /// Validation to apply when condition is met
    pub validator: Option<Arc<dyn Fn(&T, &ValidationContext) -> bool + Send + Sync>>,
    /// Whether field is required when condition is met
    pub required_when_met: bool,
    /// Custom error message
    pub message: Option<String>,
}

impl<T: ?Sized> DependencyRule<T> {
    /// Create a new dependency rule.
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
            depends_on_field: None,
            condition: None,
            validator: None,
            required_when_met: false,
            message: None,
        }
    }

    /// Set the field this depends on and condition.
    pub fn depends_on(mut self, field: impl Into<String>, condition: DependencyCondition) -> Self {
        self.depends_on_field = Some(field.into());
        self.condition = Some(condition);
        self
    }

    /// Make field required when condition is met.
    pub fn then_required(mut self) -> Self {
        self.required_when_met = true;
        self
    }

    /// Set custom validation when condition is met.
    pub fn then_validate<F>(mut self, validator: F) -> Self
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
            if let Some(ref dep_field) = self.depends_on_field {
                format!("Field '{}' is required based on '{}'", self.field_name, dep_field)
            } else {
                format!("Field '{}' validation failed", self.field_name)
            }
        })
    }

    /// Check if dependency condition is met
    fn check_condition(&self, ctx: &ValidationContext) -> bool {
        let Some(ref dep_field) = self.depends_on_field else {
            return false;
        };

        let Some(ref condition) = self.condition else {
            return false;
        };

        let dep_value = ctx.get_string(dep_field);

        match condition {
            DependencyCondition::Present => dep_value.map_or(false, |v| !v.is_empty()),
            DependencyCondition::Absent => dep_value.map_or(true, |v| v.is_empty()),
            DependencyCondition::Equals(expected) => dep_value.map_or(false, |v| v == expected),
            DependencyCondition::NotEquals(expected) => dep_value.map_or(true, |v| v != expected),
            DependencyCondition::In(values) => {
                dep_value.map_or(false, |v| values.iter().any(|expected| v == expected))
            }
            DependencyCondition::NotIn(values) => {
                dep_value.map_or(true, |v| !values.iter().any(|expected| v == expected))
            }
        }
    }
}

impl<T: ?Sized> std::fmt::Debug for DependencyRule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependencyRule")
            .field("field_name", &self.field_name)
            .field("depends_on_field", &self.depends_on_field)
            .field("condition", &self.condition)
            .field("required_when_met", &self.required_when_met)
            .finish()
    }
}

impl Rule<str> for DependencyRule<str> {
    fn validate(&self, value: &str, ctx: &ValidationContext) -> ValidationResult<()> {
        // Check if dependency condition is met
        if !self.check_condition(ctx) {
            // Condition not met, skip validation
            return Ok(());
        }

        // Condition is met, apply validation
        if self.required_when_met && value.trim().is_empty() {
            return Err(ValidationErrors::from_iter([
                ValidationError::root("dependency.required", self.get_message())
            ]));
        }

        // Run custom validator if set
        if let Some(ref validator) = self.validator {
            if !validator(value, ctx) {
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("dependency.custom", self.get_message())
                ]));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "dependency"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl Rule<String> for DependencyRule<String> {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        // Check if dependency condition is met
        if !self.check_condition(ctx) {
            return Ok(());
        }

        if self.required_when_met && value.trim().is_empty() {
            return Err(ValidationErrors::from_iter([
                ValidationError::root("dependency.required", self.get_message())
            ]));
        }

        if let Some(ref validator) = self.validator {
            if !validator(value, ctx) {
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("dependency.custom", self.get_message())
                ]));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "dependency"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depends_on_equals() {
        let rule = DependencyRule::<str>::new("shipping_address")
            .depends_on("shipping_method", DependencyCondition::Equals("delivery".to_string()))
            .then_required();

        // When shipping_method is "delivery", address is required
        let delivery_ctx = ValidationContext::new()
            .with_field("shipping_method", "delivery");
        
        assert!(rule.validate("", &delivery_ctx).is_err()); // Empty = error
        assert!(rule.validate("123 Main St", &delivery_ctx).is_ok());

        // When shipping_method is "pickup", address is optional
        let pickup_ctx = ValidationContext::new()
            .with_field("shipping_method", "pickup");
        
        assert!(rule.validate("", &pickup_ctx).is_ok()); // Empty = ok
    }

    #[test]
    fn test_depends_on_present() {
        let rule = DependencyRule::<str>::new("postal_code")
            .depends_on("country", DependencyCondition::Present)
            .then_required();

        let with_country = ValidationContext::new()
            .with_field("country", "US");
        
        assert!(rule.validate("", &with_country).is_err());

        let without_country = ValidationContext::new();
        assert!(rule.validate("", &without_country).is_ok());
    }

    #[test]
    fn test_depends_on_in() {
        let rule = DependencyRule::<str>::new("state")
            .depends_on("country", DependencyCondition::In(vec!["US".to_string(), "CA".to_string()]))
            .then_required();

        let us_ctx = ValidationContext::new().with_field("country", "US");
        assert!(rule.validate("", &us_ctx).is_err());

        let uk_ctx = ValidationContext::new().with_field("country", "UK");
        assert!(rule.validate("", &uk_ctx).is_ok()); // UK not in list, skip
    }
}
