//! Whitespace transformation rules (trim).

use skp_validator_core::Transform;

/// Trim transformation - removes leading and trailing whitespace.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::transform::whitespace::TrimTransform;
/// use skp_validator_core::Transform;
///
/// let transform = TrimTransform;
/// assert_eq!(transform.transform("  hello  ".to_string()), "hello");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TrimTransform;

impl Transform<String> for TrimTransform {
    fn transform(&self, value: String) -> String {
        value.trim().to_string()
    }

    fn name(&self) -> &'static str {
        "trim"
    }
}

/// Trim start transformation - removes leading whitespace.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrimStartTransform;

impl Transform<String> for TrimStartTransform {
    fn transform(&self, value: String) -> String {
        value.trim_start().to_string()
    }

    fn name(&self) -> &'static str {
        "trim_start"
    }
}

/// Trim end transformation - removes trailing whitespace.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrimEndTransform;

impl Transform<String> for TrimEndTransform {
    fn transform(&self, value: String) -> String {
        value.trim_end().to_string()
    }

    fn name(&self) -> &'static str {
        "trim_end"
    }
}

/// Collapse whitespace transformation - replaces multiple spaces with single space.
#[derive(Debug, Clone, Copy, Default)]
pub struct CollapseWhitespaceTransform;

impl Transform<String> for CollapseWhitespaceTransform {
    fn transform(&self, value: String) -> String {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn name(&self) -> &'static str {
        "collapse_whitespace"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() {
        let transform = TrimTransform;
        assert_eq!(transform.transform("  hello  ".to_string()), "hello");
        assert_eq!(transform.transform("\t\nhello\n\t".to_string()), "hello");
        assert_eq!(transform.transform("hello".to_string()), "hello");
    }

    #[test]
    fn test_trim_start() {
        let transform = TrimStartTransform;
        assert_eq!(transform.transform("  hello  ".to_string()), "hello  ");
    }

    #[test]
    fn test_trim_end() {
        let transform = TrimEndTransform;
        assert_eq!(transform.transform("  hello  ".to_string()), "  hello");
    }

    #[test]
    fn test_collapse_whitespace() {
        let transform = CollapseWhitespaceTransform;
        assert_eq!(transform.transform("hello   world".to_string()), "hello world");
        assert_eq!(transform.transform("  hello   world  ".to_string()), "hello world");
    }
}
