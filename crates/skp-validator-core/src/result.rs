//! Validation result types.

use crate::error::ValidationErrors;

/// Result type for validation operations.
///
/// Returns `Ok(T)` on success, or `Err(ValidationErrors)` containing
/// all validation failures.
pub type ValidationResult<T> = Result<T, ValidationErrors>;

/// Extension trait for ValidationResult
pub trait ValidationResultExt<T> {
    /// Map the success value, preserving validation errors
    fn map_valid<U, F: FnOnce(T) -> U>(self, f: F) -> ValidationResult<U>;

    /// Combine with another validation result, collecting all errors
    fn and_also<U>(self, other: ValidationResult<U>) -> ValidationResult<(T, U)>;

    /// Add errors from another result without changing the success type
    fn with_errors(self, other: ValidationResult<()>) -> ValidationResult<T>;
}

impl<T> ValidationResultExt<T> for ValidationResult<T> {
    fn map_valid<U, F: FnOnce(T) -> U>(self, f: F) -> ValidationResult<U> {
        self.map(f)
    }

    fn and_also<U>(self, other: ValidationResult<U>) -> ValidationResult<(T, U)> {
        match (self, other) {
            (Ok(t), Ok(u)) => Ok((t, u)),
            (Err(mut e1), Err(e2)) => {
                e1.merge(e2);
                Err(e1)
            }
            (Err(e), Ok(_)) | (Ok(_), Err(e)) => Err(e),
        }
    }

    fn with_errors(self, other: ValidationResult<()>) -> ValidationResult<T> {
        match (self, other) {
            (Ok(t), Ok(())) => Ok(t),
            (Err(mut e1), Err(e2)) => {
                e1.merge(e2);
                Err(e1)
            }
            (Err(e), Ok(())) => Err(e),
            (Ok(_), Err(e)) => Err(e),
        }
    }
}

/// Helper for collecting multiple validation results
pub struct ValidationCollector {
    errors: ValidationErrors,
}

impl ValidationCollector {
    /// Create a new collector
    pub fn new() -> Self {
        Self {
            errors: ValidationErrors::new(),
        }
    }

    /// Add a validation result, collecting any errors
    pub fn collect<T>(&mut self, result: ValidationResult<T>) -> Option<T> {
        match result {
            Ok(v) => Some(v),
            Err(e) => {
                self.errors.merge(e);
                None
            }
        }
    }

    /// Add a validation result for a specific field
    pub fn collect_field<T>(
        &mut self,
        field: impl Into<String>,
        result: ValidationResult<T>,
    ) -> Option<T> {
        match result {
            Ok(v) => Some(v),
            Err(e) => {
                self.errors.add_nested_errors(field, e);
                None
            }
        }
    }

    /// Check if any errors were collected
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the collected errors
    pub fn errors(&self) -> &ValidationErrors {
        &self.errors
    }

    /// Finish collecting and return the result
    pub fn finish(self) -> ValidationResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    /// Finish with a value if no errors
    pub fn finish_with<T>(self, value: T) -> ValidationResult<T> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self.errors)
        }
    }
}

impl Default for ValidationCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValidationError;

    #[test]
    fn test_and_also_both_ok() {
        let r1: ValidationResult<i32> = Ok(1);
        let r2: ValidationResult<&str> = Ok("test");
        let combined = r1.and_also(r2);
        assert_eq!(combined.unwrap(), (1, "test"));
    }

    #[test]
    fn test_and_also_both_err() {
        let mut e1 = ValidationErrors::new();
        e1.add_field_error("a", ValidationError::new("a", "code", "Error A"));

        let mut e2 = ValidationErrors::new();
        e2.add_field_error("b", ValidationError::new("b", "code", "Error B"));

        let r1: ValidationResult<i32> = Err(e1);
        let r2: ValidationResult<&str> = Err(e2);
        let combined = r1.and_also(r2);

        let errors = combined.unwrap_err();
        assert_eq!(errors.count(), 2);
    }

    #[test]
    fn test_collector() {
        let mut collector = ValidationCollector::new();

        let r1: ValidationResult<i32> = Ok(42);
        let v1 = collector.collect(r1);
        assert_eq!(v1, Some(42));

        let mut e = ValidationErrors::new();
        e.add_field_error("x", ValidationError::new("x", "code", "Error"));
        let r2: ValidationResult<i32> = Err(e);
        let v2 = collector.collect(r2);
        assert_eq!(v2, None);

        assert!(collector.has_errors());
        assert!(collector.finish().is_err());
    }
}
