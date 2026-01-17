//! Unique items validation rule for collections.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use std::collections::HashSet;
use std::hash::Hash;

/// Unique items validation rule - all items in a collection must be unique.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::collection::unique_items::UniqueItemsRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = UniqueItemsRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate(&vec!["a", "b", "c"], &ctx).is_ok());
/// assert!(rule.validate(&vec!["a", "b", "a"], &ctx).is_err());
/// ```
#[derive(Debug, Clone, Default)]
pub struct UniqueItemsRule {
    /// Custom error message
    pub message: Option<String>,
}

impl UniqueItemsRule {
    /// Create a new unique_items rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "All items must be unique".to_string())
    }
}

impl<T: Eq + Hash> Rule<Vec<T>> for UniqueItemsRule {
    fn validate(&self, value: &Vec<T>, _ctx: &ValidationContext) -> ValidationResult<()> {
        let mut seen = HashSet::new();
        let mut duplicates = 0;

        for item in value {
            if !seen.insert(item) {
                duplicates += 1;
            }
        }

        if duplicates == 0 {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "unique_items", self.get_message())
                    .with_param("duplicate_count", duplicates as i64)
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "unique_items"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

impl<T: Eq + Hash> Rule<[T]> for UniqueItemsRule {
    fn validate(&self, value: &[T], _ctx: &ValidationContext) -> ValidationResult<()> {
        let mut seen = HashSet::new();
        let mut duplicates = 0;

        for item in value {
            if !seen.insert(item) {
                duplicates += 1;
            }
        }

        if duplicates == 0 {
            Ok(())
        } else {
            Err(ValidationErrors::from_iter([
                ValidationError::new("", "unique_items", self.get_message())
                    .with_param("duplicate_count", duplicates as i64)
            ]))
        }
    }

    fn name(&self) -> &'static str {
        "unique_items"
    }

    fn default_message(&self) -> String {
        self.get_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_strings() {
        let rule = UniqueItemsRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate(&vec!["a", "b", "c"], &ctx).is_ok());
        assert!(rule.validate(&vec!["a", "b", "a"], &ctx).is_err());
    }

    #[test]
    fn test_unique_numbers() {
        let rule = UniqueItemsRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate(&vec![1, 2, 3, 4, 5], &ctx).is_ok());
        assert!(rule.validate(&vec![1, 2, 3, 2, 5], &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = UniqueItemsRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate(&Vec::<i32>::new(), &ctx).is_ok());
    }

    #[test]
    fn test_single_item() {
        let rule = UniqueItemsRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate(&vec![1], &ctx).is_ok());
    }
}
