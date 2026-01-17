//! Dependency validation example
//!
//! This example demonstrates conditional validation where field validation
//! depends on the values of other fields using FieldDependency.

use oc_validator::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Account opening form with complex field dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountOpeningForm {
    /// Account type determines other field requirements
    pub account_type: String,
    /// Employment type affects employer requirements
    pub employment_type: Option<String>,
    /// Employer name required only for salaried employees
    pub employer_name: Option<String>,
    /// Minimum balance depends on account type
    pub minimum_balance: Option<f64>,
    /// Credit card required only for VIP accounts
    pub credit_card: Option<String>,
    /// VIP status affects credit card requirements
    pub is_vip: bool,
}

impl AccountOpeningForm {
    /// Create validation configuration with dependencies
    pub fn validation_config() -> ValidationConfig {
        let mut config = ValidationConfig::new();

        // Account type validation
        let account_config = FieldValidationConfig::new("account_type")
            .add_rule(ValidationRule::Required {
                message: Some("Account type is required".to_string()),
            })
            .add_rule(ValidationRule::AllowedValues {
                values: vec![
                    "SAVINGS".to_string(),
                    "CURRENT".to_string(),
                    "TRADING".to_string(),
                ],
                message: None,
            })
            .add_rule(ValidationRule::Uppercase);

        config = config.add_field(account_config);

        // Employment type validation
        let employment_config = FieldValidationConfig::new("employment_type")
            .add_rule(ValidationRule::AllowedValues {
                values: vec![
                    "SALARIED".to_string(),
                    "SELF_EMPLOYED".to_string(),
                    "BUSINESS".to_string(),
                ],
                message: None,
            })
            .add_rule(ValidationRule::Uppercase);

        config = config.add_field(employment_config);

        // Employer name dependency - required only for salaried employees
        let employer_dependency = FieldDependency::new(
            "employer_name",
            "employment_type",
            Arc::new(|ctx| {
                ctx.get_string("employment_type")
                    .map(|et| et == "SALARIED")
                    .unwrap_or(false)
            }),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Employer name is required for salaried employees".to_string()),
        })
        .add_rule(ValidationRule::Length {
            min: Some(2),
            max: Some(100),
            message: None,
        })
        .with_description("Employer details are required when employment type is salaried");

        config = config.add_dependency(employer_dependency);

        // Minimum balance dependency - different requirements per account type
        let balance_dependency = FieldDependency::new(
            "minimum_balance",
            "account_type",
            Arc::new(|ctx| ctx.has_value("account_type")),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Minimum balance is required".to_string()),
        })
        .add_rule(ValidationRule::Contextual {
            validator: Arc::new(|ctx| {
                let account_type = ctx.get_string("account_type").unwrap_or("");
                let balance = ctx.get_number("minimum_balance").unwrap_or(0.0);

                let min_required = match account_type {
                    "SAVINGS" => 1000.0,
                    "CURRENT" => 5000.0,
                    "TRADING" => 25000.0,
                    _ => 0.0,
                };

                if balance < min_required {
                    return Err(vec![FieldError::new(
                        "minimum_balance",
                        format!(
                            "{} account requires minimum ₹{:.0}",
                            account_type, min_required
                        ),
                    )]);
                }

                Ok(())
            }),
            message: None,
        })
        .with_description("Minimum balance requirements vary by account type");

        config = config.add_dependency(balance_dependency);

        // Credit card dependency - required only for VIP customers
        let credit_card_dependency = FieldDependency::new(
            "credit_card",
            "is_vip",
            Arc::new(|ctx| ctx.get_bool("is_vip").unwrap_or(false)),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Credit card is required for VIP customers".to_string()),
        })
        .add_rule(ValidationRule::Regex {
            pattern: r"^\d{13,19}$".to_string(),
            message: Some("Credit card number must be 13-19 digits".to_string()),
        })
        .with_description("VIP customers must provide credit card information");

        config = config.add_dependency(credit_card_dependency);

        config
    }

    /// Validate the form using the dependency configuration
    pub fn validate_form(&self) -> Result<ValidationResult<()>, serde_json::Error> {
        let config = Self::validation_config();
        let mut context = ValidationContext::from_serde(self)?;
        Ok(config.validate(&mut context))
    }
}

/// Order form with payment method dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderForm {
    pub order_type: String,
    pub payment_method: String,
    pub card_number: Option<String>,
    pub upi_id: Option<String>,
    pub bank_account: Option<String>,
    pub total_amount: f64,
}

impl OrderForm {
    pub fn validation_config() -> ValidationConfig {
        let mut config = ValidationConfig::new();

        // Order type
        config = config.add_field(
            FieldValidationConfig::new("order_type")
                .add_rule(ValidationRule::Required { message: None })
                .add_rule(ValidationRule::AllowedValues {
                    values: vec!["PHYSICAL".to_string(), "DIGITAL".to_string()],
                    message: None,
                })
                .add_rule(ValidationRule::Uppercase),
        );

        // Payment method
        config = config.add_field(
            FieldValidationConfig::new("payment_method")
                .add_rule(ValidationRule::Required { message: None })
                .add_rule(ValidationRule::AllowedValues {
                    values: vec!["CARD".to_string(), "UPI".to_string(), "BANK".to_string()],
                    message: None,
                })
                .add_rule(ValidationRule::Uppercase),
        );

        // Card number - required only for card payments
        let card_dependency = FieldDependency::new(
            "card_number",
            "payment_method",
            Arc::new(|ctx| ctx.get_string("payment_method").unwrap_or("") == "CARD"),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Card number is required for card payments".to_string()),
        })
        .add_rule(ValidationRule::Regex {
            pattern: r"^\d{13,19}$".to_string(),
            message: Some("Invalid card number format".to_string()),
        });

        config = config.add_dependency(card_dependency);

        // UPI ID - required only for UPI payments
        let upi_dependency = FieldDependency::new(
            "upi_id",
            "payment_method",
            Arc::new(|ctx| ctx.get_string("payment_method").unwrap_or("") == "UPI"),
        )
        .add_rule(ValidationRule::Required {
            message: Some("UPI ID is required for UPI payments".to_string()),
        })
        .add_rule(ValidationRule::Regex {
            pattern: r"^[a-zA-Z0-9._-]+@[a-zA-Z0-9.-]+$".to_string(),
            message: Some("Invalid UPI ID format".to_string()),
        });

        config = config.add_dependency(upi_dependency);

        // Bank account - required only for bank transfers
        let bank_dependency = FieldDependency::new(
            "bank_account",
            "payment_method",
            Arc::new(|ctx| ctx.get_string("payment_method").unwrap_or("") == "BANK"),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Bank account is required for bank transfers".to_string()),
        })
        .add_rule(ValidationRule::Length {
            min: Some(8),
            max: Some(20),
            message: Some("Bank account number must be 8-20 characters".to_string()),
        });

        config = config.add_dependency(bank_dependency);

        config
    }

    pub fn validate_order(&self) -> Result<ValidationResult<()>, serde_json::Error> {
        let config = Self::validation_config();
        let mut context = ValidationContext::from_serde(self)?;
        Ok(config.validate(&mut context))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 OneClick Validator - Dependency Validation Example");
    println!("=================================================\n");

    // Test account opening with dependencies
    println!("🏦 Testing account opening dependencies:");

    // Valid salaried employee with savings account
    println!("\n✅ Valid salaried employee with savings account:");
    let valid_account = AccountOpeningForm {
        account_type: "savings".to_string(),
        employment_type: Some("salaried".to_string()),
        employer_name: Some("Tech Corp".to_string()),
        minimum_balance: Some(5000.0),
        credit_card: None, // Not required for non-VIP
        is_vip: false,
    };

    match valid_account.validate_form() {
        Ok(Ok(_)) => println!("   ✓ Account opening form is valid"),
        Ok(Err(errors)) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Valid VIP with credit card
    println!("\n✅ Valid VIP customer with credit card:");
    let vip_account = AccountOpeningForm {
        account_type: "trading".to_string(),
        employment_type: Some("self_employed".to_string()),
        employer_name: None, // Not required for self-employed
        minimum_balance: Some(50000.0),
        credit_card: Some("1234567890123456".to_string()),
        is_vip: true,
    };

    match vip_account.validate_form() {
        Ok(Ok(_)) => println!("   ✓ VIP account opening form is valid"),
        Ok(Err(errors)) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Invalid cases
    println!("\n❌ Testing invalid dependency scenarios:");

    // Salaried employee without employer name
    let invalid_employer = AccountOpeningForm {
        account_type: "savings".to_string(),
        employment_type: Some("salaried".to_string()),
        employer_name: None, // Missing required field
        minimum_balance: Some(5000.0),
        credit_card: None,
        is_vip: false,
    };

    match invalid_employer.validate_form() {
        Ok(Ok(_)) => println!("   ✗ Unexpected: Missing employer name passed validation"),
        Ok(Err(errors)) => {
            println!("   ✓ Correctly failed: Missing employer name for salaried employee");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Insufficient minimum balance
    let insufficient_balance = AccountOpeningForm {
        account_type: "trading".to_string(),
        employment_type: Some("business".to_string()),
        employer_name: None,
        minimum_balance: Some(5000.0), // Too low for trading account
        credit_card: None,
        is_vip: false,
    };

    match insufficient_balance.validate_form() {
        Ok(Ok(_)) => println!("   ✗ Unexpected: Insufficient balance passed validation"),
        Ok(Err(errors)) => {
            println!("   ✓ Correctly failed: Insufficient minimum balance for trading account");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // VIP without credit card
    let vip_no_card = AccountOpeningForm {
        account_type: "current".to_string(),
        employment_type: Some("business".to_string()),
        employer_name: None,
        minimum_balance: Some(10000.0),
        credit_card: None, // Missing required field for VIP
        is_vip: true,
    };

    match vip_no_card.validate_form() {
        Ok(Ok(_)) => println!("   ✗ Unexpected: VIP without credit card passed validation"),
        Ok(Err(errors)) => {
            println!("   ✓ Correctly failed: VIP customer missing credit card");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Test order form dependencies
    println!("\n🛒 Testing order form payment dependencies:");

    // Valid card payment
    let card_order = OrderForm {
        order_type: "physical".to_string(),
        payment_method: "card".to_string(),
        card_number: Some("4111111111111111".to_string()),
        upi_id: None,
        bank_account: None,
        total_amount: 99.99,
    };

    match card_order.validate_order() {
        Ok(Ok(_)) => println!("   ✓ Card payment order is valid"),
        Ok(Err(errors)) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Valid UPI payment
    let upi_order = OrderForm {
        order_type: "digital".to_string(),
        payment_method: "upi".to_string(),
        card_number: None,
        upi_id: Some("user@paytm".to_string()),
        bank_account: None,
        total_amount: 49.99,
    };

    match upi_order.validate_order() {
        Ok(Ok(_)) => println!("   ✓ UPI payment order is valid"),
        Ok(Err(errors)) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Invalid - card payment without card number
    let invalid_card_order = OrderForm {
        order_type: "physical".to_string(),
        payment_method: "card".to_string(),
        card_number: None, // Missing required field
        upi_id: None,
        bank_account: None,
        total_amount: 99.99,
    };

    match invalid_card_order.validate_order() {
        Ok(Ok(_)) => {
            println!("   ✗ Unexpected: Card payment without card number passed validation")
        }
        Ok(Err(errors)) => {
            println!("   ✓ Correctly failed: Card payment missing card number");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    println!();
    println!("🎉 Dependency validation example completed!");
    println!("\n💡 Key Points:");
    println!("   • Dependencies allow conditional validation based on other fields");
    println!("   • Use closures to define complex dependency conditions");
    println!("   • Dependencies are checked after field transformations");
    println!("   • Each dependency can have multiple validation rules");
    println!("   • Dependencies are reusable across different configurations");

    Ok(())
}
