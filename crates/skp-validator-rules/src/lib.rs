//! # skp-validator-rules
//!
//! Built-in validation rules for skp-validator.
//!
//! This crate provides all the standard validators organized by category:
//!
//! - **String**: email, url, ip, uuid, phone, pattern, length, ascii, alphanumeric, contains
//! - **Numeric**: range, multiple_of
//! - **Collection**: length, unique_items
//! - **Temporal**: date, date_range, age
//! - **Comparison**: required, must_match, allowed_values
//! - **Financial**: credit_card
//! - **Transform**: uppercase, lowercase, trim, capitalize
//! - **Custom**: custom, contextual, dependency

pub mod string;
pub mod numeric;
pub mod collection;
pub mod temporal;
pub mod comparison;
pub mod financial;
pub mod transform;
pub mod custom;

// Re-export commonly used rules
pub use comparison::required::RequiredRule;
pub use comparison::must_match::MustMatchRule;
pub use comparison::allowed_values::AllowedValuesRule;
pub use string::length::LengthRule;
pub use string::ascii::AsciiRule;
pub use string::alphanumeric::AlphanumericRule;
pub use string::contains::{ContainsRule, PrefixRule, SuffixRule};
pub use numeric::range::RangeRule;
pub use numeric::multiple_of::MultipleOfRule;
pub use collection::unique_items::UniqueItemsRule;
pub use financial::credit_card::CreditCardRule;
pub use transform::case::{UppercaseTransform, LowercaseTransform, CapitalizeTransform};
pub use transform::whitespace::{TrimTransform, TrimStartTransform, TrimEndTransform, CollapseWhitespaceTransform};
pub use custom::custom_fn::{CustomFnRule, CustomResultRule};
pub use custom::contextual::ContextualRule;
pub use custom::dependency::{DependencyRule, DependencyCondition};

#[cfg(feature = "email")]
pub use string::email::EmailRule;

#[cfg(feature = "regex")]
pub use string::pattern::PatternRule;

#[cfg(feature = "url")]
pub use string::url::UrlRule;

#[cfg(feature = "ip")]
pub use string::ip::{IpRule, IpVersion};

#[cfg(feature = "uuid")]
pub use string::uuid_rule::UuidRule;

#[cfg(feature = "phone")]
pub use string::phone::PhoneRule;

#[cfg(feature = "chrono")]
pub use temporal::date::DateRule;

#[cfg(feature = "chrono")]
pub use temporal::date_range::DateRangeRule;

#[cfg(feature = "chrono")]
pub use temporal::age::AgeRule;

/// Prelude with all commonly used rules
pub mod prelude {
    pub use crate::comparison::required::RequiredRule;
    pub use crate::comparison::must_match::MustMatchRule;
    pub use crate::comparison::allowed_values::AllowedValuesRule;
    pub use crate::string::length::LengthRule;
    pub use crate::string::ascii::AsciiRule;
    pub use crate::string::alphanumeric::AlphanumericRule;
    pub use crate::string::contains::{ContainsRule, PrefixRule, SuffixRule};
    pub use crate::numeric::range::RangeRule;
    pub use crate::numeric::multiple_of::MultipleOfRule;
    pub use crate::collection::unique_items::UniqueItemsRule;
    pub use crate::financial::credit_card::CreditCardRule;
    pub use crate::transform::case::{UppercaseTransform, LowercaseTransform, CapitalizeTransform};
    pub use crate::transform::whitespace::{TrimTransform, TrimStartTransform, TrimEndTransform};
    pub use crate::custom::custom_fn::{CustomFnRule, CustomResultRule};
    pub use crate::custom::contextual::ContextualRule;
    pub use crate::custom::dependency::{DependencyRule, DependencyCondition};
    
    #[cfg(feature = "email")]
    pub use crate::string::email::EmailRule;
    
    #[cfg(feature = "regex")]
    pub use crate::string::pattern::PatternRule;
    
    #[cfg(feature = "url")]
    pub use crate::string::url::UrlRule;
    
    #[cfg(feature = "ip")]
    pub use crate::string::ip::{IpRule, IpVersion};

    #[cfg(feature = "uuid")]
    pub use crate::string::uuid_rule::UuidRule;

    #[cfg(feature = "phone")]
    pub use crate::string::phone::PhoneRule;

    #[cfg(feature = "chrono")]
    pub use crate::temporal::date::DateRule;

    #[cfg(feature = "chrono")]
    pub use crate::temporal::date_range::DateRangeRule;

    #[cfg(feature = "chrono")]
    pub use crate::temporal::age::AgeRule;
}
