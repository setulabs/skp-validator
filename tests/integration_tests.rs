use oc_validator::prelude::*;
use oc_validator::{
    FieldDependency, FieldValidationConfig, ValidationConfig, ValidationContext, ValidationRule,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Example struct with basic validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TestData {
    #[validate]
    pub required_field: String,

    pub optional_field: Option<String>,
}

/// Example struct with advanced validation features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTestData {
    pub name: String,
    pub email: String,
    pub age: Option<u32>,
    pub phone: Option<String>,
    pub date_of_birth: String,
    pub income: Option<f64>,
    pub profile_type: String,
    pub is_vip: bool,
    pub credit_card: Option<String>,
}

impl AdvancedTestData {
    pub fn validation_config() -> ValidationConfig {
        let mut config = ValidationConfig::new();

        // Name validation with length and transformation
        let name_config = FieldValidationConfig::new("name")
            .add_rule(ValidationRule::Required {
                message: Some("Name is required".to_string()),
            })
            .add_rule(ValidationRule::Length {
                min: Some(2),
                max: Some(50),
                message: None,
            })
            .add_rule(ValidationRule::Uppercase);
        config = config.add_field(name_config);

        // Email validation
        let email_config = FieldValidationConfig::new("email")
            .add_rule(ValidationRule::Required { message: None })
            .add_rule(ValidationRule::Email { message: None });
        config = config.add_field(email_config);

        // Age validation with range
        let age_config = FieldValidationConfig::new("age").add_rule(ValidationRule::Range {
            min: Some(18.0),
            max: Some(120.0),
            message: None,
        });
        config = config.add_field(age_config);

        // Phone validation
        let phone_config =
            FieldValidationConfig::new("phone").add_rule(ValidationRule::Phone { message: None });
        config = config.add_field(phone_config);

        // Date of birth validation
        let dob_config = FieldValidationConfig::new("date_of_birth")
            .add_rule(ValidationRule::Required { message: None })
            .add_rule(ValidationRule::Date {
                format: "%d-%m-%Y".to_string(),
                message: None,
            })
            .add_rule(ValidationRule::Age {
                min: Some(18),
                max: Some(100),
                date_format: "%d-%m-%Y".to_string(),
                message: None,
            });
        config = config.add_field(dob_config);

        // Income validation with dependency
        let income_config = FieldValidationConfig::new("income").add_rule(ValidationRule::Range {
            min: Some(0.0),
            max: None,
            message: None,
        });
        config = config.add_field(income_config);

        // Profile type validation
        let profile_config = FieldValidationConfig::new("profile_type")
            .add_rule(ValidationRule::Required { message: None })
            .add_rule(ValidationRule::AllowedValues {
                values: vec![
                    "BASIC".to_string(),
                    "PREMIUM".to_string(),
                    "VIP".to_string(),
                ],
                message: None,
            })
            .add_rule(ValidationRule::Uppercase);
        config = config.add_field(profile_config);

        // Credit card validation - only required for VIP users
        let credit_card_dependency = FieldDependency::new(
            "credit_card",
            "profile_type",
            Arc::new(|ctx| {
                ctx.get_string("profile_type")
                    .map(|pt| pt == "VIP")
                    .unwrap_or(false)
            }),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Credit card required for VIP users".to_string()),
        })
        .with_description("Credit card is required when profile type is VIP");

        config = config.add_dependency(credit_card_dependency);

        config
    }

    pub fn validate_advanced(&self) -> ValidationResult<()> {
        let config = Self::validation_config();
        let mut context = ValidationContext::from_serde(self).map_err(|e| {
            vec![FieldError::new(
                "serialization",
                format!("Serialization error: {}", e),
            )]
        })?;
        config.validate(&mut context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        // Valid data
        let valid_data = TestData {
            required_field: "test".to_string(),
            optional_field: Some("optional".to_string()),
        };
        assert!(valid_data.validate().is_ok());

        // Invalid data - required field empty
        let invalid_data = TestData {
            required_field: "".to_string(),
            optional_field: None,
        };
        let errors = invalid_data.validate().unwrap_err();
        assert!(!errors.is_empty());
        println!("Validation errors: {:?}", errors);
    }

    #[test]
    fn test_date_validation() {
        // Test date format validation
        let mut config = ValidationConfig::new();
        let date_config = FieldValidationConfig::new("test_date").add_rule(ValidationRule::Date {
            format: "%Y-%m-%d".to_string(),
            message: None,
        });
        config = config.add_field(date_config);

        // Valid date
        let mut context = ValidationContext::new();
        context.set_field_value(
            "test_date",
            serde_json::Value::String("2023-12-25".to_string()),
        );
        assert!(config.validate(&mut context).is_ok());

        // Invalid date
        let mut context = ValidationContext::new();
        context.set_field_value(
            "test_date",
            serde_json::Value::String("invalid-date".to_string()),
        );
        assert!(config.validate(&mut context).is_err());
    }

    #[test]
    fn test_age_validation() {
        let mut config = ValidationConfig::new();
        let age_config = FieldValidationConfig::new("dob").add_rule(ValidationRule::Age {
            min: Some(18),
            max: Some(65),
            date_format: "%Y-%m-%d".to_string(),
            message: None,
        });
        config = config.add_field(age_config);

        // Valid age (30 years old)
        let mut context = ValidationContext::new();
        context.set_field_value("dob", serde_json::Value::String("1993-12-25".to_string()));
        assert!(config.validate(&mut context).is_ok());

        // Invalid age (too young)
        let mut context = ValidationContext::new();
        context.set_field_value("dob", serde_json::Value::String("2010-12-25".to_string()));
        assert!(config.validate(&mut context).is_err());
    }

    #[test]
    fn test_field_transformation() {
        let mut config = ValidationConfig::new();
        let transform_config =
            FieldValidationConfig::new("test_field").add_rule(ValidationRule::Uppercase);
        config = config.add_field(transform_config);

        let mut context = ValidationContext::new();
        context.set_field_value(
            "test_field",
            serde_json::Value::String("hello world".to_string()),
        );

        // Should transform to uppercase
        assert!(config.validate(&mut context).is_ok());
        assert_eq!(context.get_string("test_field"), Some("HELLO WORLD"));
    }

    #[test]
    fn test_dependency_validation() {
        let mut config = ValidationConfig::new();

        // Basic field
        let profile_config = FieldValidationConfig::new("profile_type")
            .add_rule(ValidationRule::Required { message: None });
        config = config.add_field(profile_config);

        // Dependency: credit_card required when profile_type is "VIP"
        let dependency = FieldDependency::new(
            "credit_card",
            "profile_type",
            Arc::new(|ctx| {
                ctx.get_string("profile_type")
                    .map(|pt| pt == "VIP")
                    .unwrap_or(false)
            }),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Credit card required for VIP".to_string()),
        });
        config = config.add_dependency(dependency);

        // Test VIP user without credit card - should fail
        let mut context = ValidationContext::new();
        context.set_field_value("profile_type", serde_json::Value::String("VIP".to_string()));
        // credit_card not set
        assert!(config.validate(&mut context).is_err());

        // Test VIP user with credit card - should pass
        let mut context = ValidationContext::new();
        context.set_field_value("profile_type", serde_json::Value::String("VIP".to_string()));
        context.set_field_value(
            "credit_card",
            serde_json::Value::String("1234567890123456".to_string()),
        );
        assert!(config.validate(&mut context).is_ok());

        // Test regular user without credit card - should pass
        let mut context = ValidationContext::new();
        context.set_field_value(
            "profile_type",
            serde_json::Value::String("BASIC".to_string()),
        );
        // credit_card not set
        assert!(config.validate(&mut context).is_ok());
    }

    #[test]
    fn test_advanced_validation_example() {
        // Valid VIP user
        let vip_user = AdvancedTestData {
            name: "john doe".to_string(),
            email: "john@example.com".to_string(),
            age: Some(30),
            phone: Some("+1234567890".to_string()),
            date_of_birth: "25-12-1993".to_string(),
            income: Some(75000.0),
            profile_type: "vip".to_string(),
            is_vip: true,
            credit_card: Some("1234567890123456".to_string()),
        };

        let result = vip_user.validate_advanced();
        assert!(
            result.is_ok(),
            "VIP user validation should pass: {:?}",
            result.err()
        );

        // Invalid - missing credit card for VIP
        let invalid_vip = AdvancedTestData {
            name: "jane doe".to_string(),
            email: "jane@example.com".to_string(),
            age: Some(25),
            phone: Some("+1234567890".to_string()),
            date_of_birth: "15-03-1998".to_string(),
            income: Some(50000.0),
            profile_type: "vip".to_string(),
            is_vip: true,
            credit_card: None, // Missing credit card
        };

        let result = invalid_vip.validate_advanced();
        assert!(result.is_err(), "VIP without credit card should fail");

        // Valid basic user
        let basic_user = AdvancedTestData {
            name: "bob smith".to_string(),
            email: "bob@example.com".to_string(),
            age: Some(28),
            phone: Some("+1234567890".to_string()),
            date_of_birth: "10-05-1995".to_string(),
            income: Some(45000.0),
            profile_type: "basic".to_string(),
            is_vip: false,
            credit_card: None, // Not required for basic
        };

        let result = basic_user.validate_advanced();
        assert!(
            result.is_ok(),
            "Basic user validation should pass: {:?}",
            result.err()
        );
    }
}
