//! Length validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Mode for length calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LengthMode {
    /// Count Unicode characters (default)
    #[default]
    Chars,
    /// Count bytes
    Bytes,
    /// Count grapheme clusters (requires unicode-segmentation)
    Graphemes,
}

/// String/collection length validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::length::LengthRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = LengthRule::new().min(3).max(50);
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("hello", &ctx).is_ok());
/// assert!(rule.validate("ab", &ctx).is_err()); // Too short
/// ```
#[derive(Debug, Clone)]
pub struct LengthRule {
    /// Minimum length (inclusive)
    pub min: Option<usize>,
    /// Maximum length (inclusive)
    pub max: Option<usize>,
    /// Exact length (if set, min/max are ignored)
    pub equal: Option<usize>,
    /// Length calculation mode
    pub mode: LengthMode,
    /// Custom error message
    pub message: Option<String>,
}

impl LengthRule {
    /// Create a new length rule.
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            equal: None,
            mode: LengthMode::default(),
            message: None,
        }
    }

    /// Set minimum length.
    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum length.
    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    /// Set exact length.
    pub fn equal(mut self, len: usize) -> Self {
        self.equal = Some(len);
        self
    }

    /// Set length calculation mode.
    pub fn mode(mut self, mode: LengthMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Calculate length based on mode.
    fn calculate_length(&self, s: &str) -> usize {
        match self.mode {
            LengthMode::Chars => s.chars().count(),
            LengthMode::Bytes => s.len(),
            LengthMode::Graphemes => {
                // Simple approximation without unicode-segmentation
                // In production, use the unicode-segmentation crate
                s.chars().count()
            }
        }
    }
}

impl Default for LengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule<str> for LengthRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        let len = self.calculate_length(value);

        // Check exact length first
        if let Some(exact) = self.equal {
            if len != exact {
                let msg = self.message.clone().unwrap_or_else(|| {
                    format!("Must be exactly {} characters", exact)
                });
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("length.equal", msg)
                        .with_param("expected", exact as i64)
                        .with_param("actual", len as i64)
                ]));
            }
            return Ok(());
        }

        // Check min length
        if let Some(min) = self.min {
            if len < min {
                let msg = self.message.clone().unwrap_or_else(|| {
                    format!("Must be at least {} characters", min)
                });
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("length.min", msg)
                        .with_param("min", min as i64)
                        .with_param("actual", len as i64)
                ]));
            }
        }

        // Check max length
        if let Some(max) = self.max {
            if len > max {
                let msg = self.message.clone().unwrap_or_else(|| {
                    format!("Must be at most {} characters", max)
                });
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("length.max", msg)
                        .with_param("max", max as i64)
                        .with_param("actual", len as i64)
                ]));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "length"
    }

    fn default_message(&self) -> String {
        if let Some(exact) = self.equal {
            format!("Must be exactly {} characters", exact)
        } else {
            match (self.min, self.max) {
                (Some(min), Some(max)) => format!("Must be between {} and {} characters", min, max),
                (Some(min), None) => format!("Must be at least {} characters", min),
                (None, Some(max)) => format!("Must be at most {} characters", max),
                (None, None) => "Invalid length".to_string(),
            }
        }
    }
}

impl Rule<String> for LengthRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "length"
    }

    fn default_message(&self) -> String {
        <Self as Rule<str>>::default_message(self)
    }
}

// Implement for collections
impl<T> Rule<Vec<T>> for LengthRule {
    fn validate(&self, value: &Vec<T>, _ctx: &ValidationContext) -> ValidationResult<()> {
        let len = value.len();

        if let Some(exact) = self.equal {
            if len != exact {
                let msg = self.message.clone().unwrap_or_else(|| {
                    format!("Must have exactly {} items", exact)
                });
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("length.equal", msg)
                        .with_param("expected", exact as i64)
                        .with_param("actual", len as i64)
                ]));
            }
            return Ok(());
        }

        if let Some(min) = self.min {
            if len < min {
                let msg = self.message.clone().unwrap_or_else(|| {
                    format!("Must have at least {} items", min)
                });
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("length.min", msg)
                        .with_param("min", min as i64)
                        .with_param("actual", len as i64)
                ]));
            }
        }

        if let Some(max) = self.max {
            if len > max {
                let msg = self.message.clone().unwrap_or_else(|| {
                    format!("Must have at most {} items", max)
                });
                return Err(ValidationErrors::from_iter([
                    ValidationError::root("length.max", msg)
                        .with_param("max", max as i64)
                        .with_param("actual", len as i64)
                ]));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "length"
    }

    fn default_message(&self) -> String {
        "Invalid length".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_length() {
        let rule = LengthRule::new().min(3);
        let ctx = ValidationContext::default();

        assert!(rule.validate("abc", &ctx).is_ok());
        assert!(rule.validate("abcd", &ctx).is_ok());
        assert!(rule.validate("ab", &ctx).is_err());
    }

    #[test]
    fn test_max_length() {
        let rule = LengthRule::new().max(5);
        let ctx = ValidationContext::default();

        assert!(rule.validate("abc", &ctx).is_ok());
        assert!(rule.validate("abcde", &ctx).is_ok());
        assert!(rule.validate("abcdef", &ctx).is_err());
    }

    #[test]
    fn test_range() {
        let rule = LengthRule::new().min(3).max(5);
        let ctx = ValidationContext::default();

        assert!(rule.validate("ab", &ctx).is_err());
        assert!(rule.validate("abc", &ctx).is_ok());
        assert!(rule.validate("abcde", &ctx).is_ok());
        assert!(rule.validate("abcdef", &ctx).is_err());
    }

    #[test]
    fn test_exact() {
        let rule = LengthRule::new().equal(5);
        let ctx = ValidationContext::default();

        assert!(rule.validate("abcd", &ctx).is_err());
        assert!(rule.validate("abcde", &ctx).is_ok());
        assert!(rule.validate("abcdef", &ctx).is_err());
    }

    #[test]
    fn test_vec_length() {
        let rule = LengthRule::new().min(2).max(4);
        let ctx = ValidationContext::default();

        assert!(rule.validate(&vec![1], &ctx).is_err());
        assert!(rule.validate(&vec![1, 2], &ctx).is_ok());
        assert!(rule.validate(&vec![1, 2, 3, 4], &ctx).is_ok());
        assert!(rule.validate(&vec![1, 2, 3, 4, 5], &ctx).is_err());
    }
}
