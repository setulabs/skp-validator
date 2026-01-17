//! Numeric range validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};

/// Numeric range validation rule.
///
/// Supports inclusive and exclusive bounds (JSON Schema compatible).
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::numeric::range::RangeRule;
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = RangeRule::<f64>::new().min(0.0).max(100.0);
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate(&50.0, &ctx).is_ok());
/// assert!(rule.validate(&-1.0, &ctx).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct RangeRule<T> {
    /// Minimum value (inclusive by default)
    pub min: Option<T>,
    /// Maximum value (inclusive by default)
    pub max: Option<T>,
    /// Whether min is exclusive
    pub exclusive_min: bool,
    /// Whether max is exclusive
    pub exclusive_max: bool,
    /// Custom error message
    pub message: Option<String>,
}

impl<T> RangeRule<T> {
    /// Create a new range rule.
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            exclusive_min: false,
            exclusive_max: false,
            message: None,
        }
    }

    /// Set minimum value (inclusive).
    pub fn min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum value (inclusive).
    pub fn max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    /// Set exclusive minimum.
    pub fn exclusive_min(mut self, min: T) -> Self {
        self.min = Some(min);
        self.exclusive_min = true;
        self
    }

    /// Set exclusive maximum.
    pub fn exclusive_max(mut self, max: T) -> Self {
        self.max = Some(max);
        self.exclusive_max = true;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

impl<T> Default for RangeRule<T> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_range_rule {
    ($($t:ty),+) => {
        $(
            impl Rule<$t> for RangeRule<$t> {
                fn validate(&self, value: &$t, _ctx: &ValidationContext) -> ValidationResult<()> {
                    // Check minimum
                    if let Some(ref min) = self.min {
                        let failed = if self.exclusive_min {
                            value <= min
                        } else {
                            value < min
                        };
                        
                        if failed {
                            let msg = self.message.clone().unwrap_or_else(|| {
                                if self.exclusive_min {
                                    format!("Must be greater than {}", min)
                                } else {
                                    format!("Must be at least {}", min)
                                }
                            });
                            return Err(ValidationErrors::from_iter([
                                ValidationError::new("", "range.min", msg)
                            ]));
                        }
                    }
                    
                    // Check maximum
                    if let Some(ref max) = self.max {
                        let failed = if self.exclusive_max {
                            value >= max
                        } else {
                            value > max
                        };
                        
                        if failed {
                            let msg = self.message.clone().unwrap_or_else(|| {
                                if self.exclusive_max {
                                    format!("Must be less than {}", max)
                                } else {
                                    format!("Must be at most {}", max)
                                }
                            });
                            return Err(ValidationErrors::from_iter([
                                ValidationError::new("", "range.max", msg)
                            ]));
                        }
                    }
                    
                    Ok(())
                }

                fn name(&self) -> &'static str {
                    "range"
                }

                fn default_message(&self) -> String {
                    "Value out of range".to_string()
                }
            }
        )+
    };
}

impl_range_rule!(i8, i16, i32, i64, i128, isize);
impl_range_rule!(u8, u16, u32, u64, u128, usize);
impl_range_rule!(f32, f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_f64() {
        let rule = RangeRule::<f64>::new().min(0.0).max(100.0);
        let ctx = ValidationContext::default();

        assert!(rule.validate(&0.0, &ctx).is_ok());
        assert!(rule.validate(&50.0, &ctx).is_ok());
        assert!(rule.validate(&100.0, &ctx).is_ok());
        assert!(rule.validate(&-1.0, &ctx).is_err());
        assert!(rule.validate(&101.0, &ctx).is_err());
    }

    #[test]
    fn test_range_i32() {
        let rule = RangeRule::<i32>::new().min(18).max(120);
        let ctx = ValidationContext::default();

        assert!(rule.validate(&18, &ctx).is_ok());
        assert!(rule.validate(&30, &ctx).is_ok());
        assert!(rule.validate(&17, &ctx).is_err());
        assert!(rule.validate(&121, &ctx).is_err());
    }

    #[test]
    fn test_exclusive() {
        let rule = RangeRule::<i32>::new().exclusive_min(0).exclusive_max(10);
        let ctx = ValidationContext::default();

        assert!(rule.validate(&0, &ctx).is_err());   // 0 is excluded
        assert!(rule.validate(&1, &ctx).is_ok());
        assert!(rule.validate(&9, &ctx).is_ok());
        assert!(rule.validate(&10, &ctx).is_err());  // 10 is excluded
    }
}
