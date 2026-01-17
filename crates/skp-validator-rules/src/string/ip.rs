//! IP address validation rule.

use skp_validator_core::{Rule, ValidationContext, ValidationErrors, ValidationError, ValidationResult};
use std::net::IpAddr;

/// IP version for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IpVersion {
    /// Any IP version (v4 or v6)
    #[default]
    Any,
    /// IPv4 only
    V4,
    /// IPv6 only
    V6,
}

/// IP address validation rule.
///
/// # Example
///
/// ```rust
/// use skp_validator_rules::string::ip::{IpRule, IpVersion};
/// use skp_validator_core::{Rule, ValidationContext};
///
/// let rule = IpRule::new();
/// let ctx = ValidationContext::default();
///
/// assert!(rule.validate("192.168.1.1", &ctx).is_ok());
/// assert!(rule.validate("::1", &ctx).is_ok());
/// assert!(rule.validate("not-an-ip", &ctx).is_err());
/// ```
#[derive(Debug, Clone, Default)]
pub struct IpRule {
    /// IP version requirement
    pub version: IpVersion,
    /// Custom error message
    pub message: Option<String>,
}

impl IpRule {
    /// Create a new IP rule (any version).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a rule for IPv4 only.
    pub fn v4() -> Self {
        Self {
            version: IpVersion::V4,
            message: None,
        }
    }

    /// Create a rule for IPv6 only.
    pub fn v6() -> Self {
        Self {
            version: IpVersion::V6,
            message: None,
        }
    }

    /// Set IP version.
    pub fn version(mut self, version: IpVersion) -> Self {
        self.version = version;
        self
    }

    /// Set custom error message.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    fn get_message(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            match self.version {
                IpVersion::Any => "Must be a valid IP address".to_string(),
                IpVersion::V4 => "Must be a valid IPv4 address".to_string(),
                IpVersion::V6 => "Must be a valid IPv6 address".to_string(),
            }
        })
    }
}

impl Rule<str> for IpRule {
    fn validate(&self, value: &str, _ctx: &ValidationContext) -> ValidationResult<()> {
        // Empty is valid (use required for non-empty)
        if value.is_empty() {
            return Ok(());
        }

        match value.parse::<IpAddr>() {
            Ok(addr) => {
                match (self.version, addr) {
                    (IpVersion::Any, _) => Ok(()),
                    (IpVersion::V4, IpAddr::V4(_)) => Ok(()),
                    (IpVersion::V6, IpAddr::V6(_)) => Ok(()),
                    (IpVersion::V4, IpAddr::V6(_)) => {
                        Err(ValidationErrors::from_iter([
                            ValidationError::root("ip.version", "Expected IPv4 address, got IPv6")
                        ]))
                    }
                    (IpVersion::V6, IpAddr::V4(_)) => {
                        Err(ValidationErrors::from_iter([
                            ValidationError::root("ip.version", "Expected IPv6 address, got IPv4")
                        ]))
                    }
                }
            }
            Err(_) => {
                Err(ValidationErrors::from_iter([
                    ValidationError::root("ip", self.get_message())
                ]))
            }
        }
    }

    fn name(&self) -> &'static str {
        "ip"
    }

    fn default_message(&self) -> String {
        "Must be a valid IP address".to_string()
    }
}

impl Rule<String> for IpRule {
    fn validate(&self, value: &String, ctx: &ValidationContext) -> ValidationResult<()> {
        <Self as Rule<str>>::validate(self, value.as_str(), ctx)
    }

    fn name(&self) -> &'static str {
        "ip"
    }

    fn default_message(&self) -> String {
        "Must be a valid IP address".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ips() {
        let rule = IpRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("192.168.1.1", &ctx).is_ok());
        assert!(rule.validate("10.0.0.1", &ctx).is_ok());
        assert!(rule.validate("::1", &ctx).is_ok());
        assert!(rule.validate("2001:0db8:85a3:0000:0000:8a2e:0370:7334", &ctx).is_ok());
    }

    #[test]
    fn test_invalid_ips() {
        let rule = IpRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("not-an-ip", &ctx).is_err());
        assert!(rule.validate("256.1.1.1", &ctx).is_err());
        assert!(rule.validate("192.168.1", &ctx).is_err());
    }

    #[test]
    fn test_v4_only() {
        let rule = IpRule::v4();
        let ctx = ValidationContext::default();

        assert!(rule.validate("192.168.1.1", &ctx).is_ok());
        assert!(rule.validate("::1", &ctx).is_err());
    }

    #[test]
    fn test_v6_only() {
        let rule = IpRule::v6();
        let ctx = ValidationContext::default();

        assert!(rule.validate("::1", &ctx).is_ok());
        assert!(rule.validate("192.168.1.1", &ctx).is_err());
    }

    #[test]
    fn test_empty_is_valid() {
        let rule = IpRule::new();
        let ctx = ValidationContext::default();

        assert!(rule.validate("", &ctx).is_ok());
    }
}
