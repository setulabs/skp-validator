//! Case transformation rules (uppercase, lowercase).

use skp_validator_core::Transform;

/// Uppercase transformation - converts string to uppercase.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::transform::case::UppercaseTransform;
/// use skp_validator_core::Transform;
///
/// let transform = UppercaseTransform;
/// assert_eq!(transform.transform("hello".to_string()), "HELLO");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct UppercaseTransform;

impl Transform<String> for UppercaseTransform {
    fn transform(&self, value: String) -> String {
        value.to_uppercase()
    }

    fn name(&self) -> &'static str {
        "uppercase"
    }
}

/// Lowercase transformation - converts string to lowercase.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::transform::case::LowercaseTransform;
/// use skp_validator_core::Transform;
///
/// let transform = LowercaseTransform;
/// assert_eq!(transform.transform("HELLO".to_string()), "hello");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct LowercaseTransform;

impl Transform<String> for LowercaseTransform {
    fn transform(&self, value: String) -> String {
        value.to_lowercase()
    }

    fn name(&self) -> &'static str {
        "lowercase"
    }
}

/// Capitalize transformation - capitalizes first letter of each word.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::transform::case::CapitalizeTransform;
/// use skp_validator_core::Transform;
///
/// let transform = CapitalizeTransform;
/// assert_eq!(transform.transform("hello world".to_string()), "Hello World");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CapitalizeTransform;

impl Transform<String> for CapitalizeTransform {
    fn transform(&self, value: String) -> String {
        value
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn name(&self) -> &'static str {
        "capitalize"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uppercase() {
        let transform = UppercaseTransform;
        assert_eq!(transform.transform("hello".to_string()), "HELLO");
        assert_eq!(transform.transform("Hello World".to_string()), "HELLO WORLD");
        assert_eq!(transform.transform("123abc".to_string()), "123ABC");
    }

    #[test]
    fn test_lowercase() {
        let transform = LowercaseTransform;
        assert_eq!(transform.transform("HELLO".to_string()), "hello");
        assert_eq!(transform.transform("Hello World".to_string()), "hello world");
    }

    #[test]
    fn test_capitalize() {
        let transform = CapitalizeTransform;
        assert_eq!(transform.transform("hello world".to_string()), "Hello World");
        assert_eq!(transform.transform("HELLO WORLD".to_string()), "HELLO WORLD");
    }
}
