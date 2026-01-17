//! JSON validation example
//!
//! This example demonstrates how to validate JSON objects directly without
//! deserializing them into Rust structs first.

use oc_validator::{
    FieldDependency, FieldValidationConfig, ValidationConfig, ValidationContext, ValidationRule,
    json_validator::{validate_json, validate_json_value},
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 OneClick Validator - JSON Validation Example");
    println!("=============================================\n");

    // Create validation configuration for user data
    println!("👤 Creating user validation configuration:");
    let mut user_config = ValidationConfig::new();

    // Username validation
    user_config = user_config.add_field(
        FieldValidationConfig::new("username")
            .add_rule(ValidationRule::Required {
                message: Some("Username is required".to_string()),
            })
            .add_rule(ValidationRule::Length {
                min: Some(3),
                max: Some(20),
                message: None,
            })
            .add_rule(ValidationRule::Regex {
                pattern: r"^[a-zA-Z0-9_]+$".to_string(),
                message: Some(
                    "Username can only contain letters, numbers, and underscores".to_string(),
                ),
            }),
    );

    // Email validation
    user_config = user_config.add_field(
        FieldValidationConfig::new("email")
            .add_rule(ValidationRule::Required {
                message: Some("Email is required".to_string()),
            })
            .add_rule(ValidationRule::Email {
                message: Some("Invalid email format".to_string()),
            }),
    );

    // Age validation
    user_config = user_config.add_field(FieldValidationConfig::new("age").add_rule(
        ValidationRule::Range {
            min: Some(18.0),
            max: Some(120.0),
            message: Some("Age must be between 18 and 120".to_string()),
        },
    ));

    // Status validation with allowed values
    user_config = user_config.add_field(FieldValidationConfig::new("status").add_rule(
        ValidationRule::AllowedValues {
            values: vec![
                "active".to_string(),
                "inactive".to_string(),
                "pending".to_string(),
            ],
            message: Some("Status must be active, inactive, or pending".to_string()),
        },
    ));

    println!("   ✓ User validation configuration created");

    // Test valid JSON user
    println!("\n✅ Testing valid JSON user:");
    let valid_user_json = json!({
        "username": "john_doe123",
        "email": "john.doe@example.com",
        "age": 25,
        "status": "active",
        "metadata": {
            "signup_date": "2024-01-15",
            "source": "web"
        }
    });

    println!(
        "   Input JSON: {}",
        serde_json::to_string_pretty(&valid_user_json)?
    );

    let mut context = ValidationContext::from_json(&valid_user_json);
    match user_config.validate(&mut context) {
        Ok(_) => println!("   ✓ JSON user validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Test invalid JSON user
    println!("\n❌ Testing invalid JSON user:");
    let invalid_user_json = json!({
        "username": "u",  // Too short
        "email": "invalid-email",  // Invalid format
        "age": 150,  // Too old
        "status": "banned",  // Not in allowed values
        "extra_field": "ignored"  // Extra fields are ignored
    });

    println!(
        "   Input JSON: {}",
        serde_json::to_string_pretty(&invalid_user_json)?
    );

    let mut context = ValidationContext::from_json(&invalid_user_json);
    match user_config.validate(&mut context) {
        Ok(_) => println!("   ✗ Unexpected: Invalid JSON passed validation"),
        Err(errors) => {
            println!(
                "   ✓ Validation correctly failed with {} errors:",
                errors.len()
            );
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Test product validation with dependencies
    println!("\n📦 Testing product validation with dependencies:");

    let mut product_config = ValidationConfig::new();

    // Product name
    product_config = product_config.add_field(
        FieldValidationConfig::new("name")
            .add_rule(ValidationRule::Required { message: None })
            .add_rule(ValidationRule::Length {
                min: Some(2),
                max: Some(100),
                message: None,
            }),
    );

    // Price
    product_config = product_config.add_field(
        FieldValidationConfig::new("price")
            .add_rule(ValidationRule::Required { message: None })
            .add_rule(ValidationRule::Range {
                min: Some(0.01),
                max: Some(100000.0),
                message: None,
            }),
    );

    // Category
    product_config = product_config.add_field(
        FieldValidationConfig::new("category")
            .add_rule(ValidationRule::Required { message: None })
            .add_rule(ValidationRule::AllowedValues {
                values: vec![
                    "electronics".to_string(),
                    "clothing".to_string(),
                    "books".to_string(),
                ],
                message: None,
            }),
    );

    // Warranty dependency - required for electronics
    let warranty_dependency = FieldDependency::new(
        "warranty_months",
        "category",
        std::sync::Arc::new(|ctx| ctx.get_string("category").unwrap_or("") == "electronics"),
    )
    .add_rule(ValidationRule::Required {
        message: Some("Warranty months are required for electronics".to_string()),
    })
    .add_rule(ValidationRule::IntRange {
        min: Some(6),
        max: Some(60),
        message: Some("Warranty must be between 6-60 months".to_string()),
    });

    product_config = product_config.add_dependency(warranty_dependency);

    // Test valid electronics product
    let valid_electronics = json!({
        "name": "Wireless Headphones",
        "price": 199.99,
        "category": "electronics",
        "warranty_months": 24
    });

    println!("   Valid electronics product:");
    println!(
        "   Input JSON: {}",
        serde_json::to_string(&valid_electronics)?
    );

    let mut context = ValidationContext::from_json(&valid_electronics);
    match product_config.validate(&mut context) {
        Ok(_) => println!("   ✓ Electronics product validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Test valid non-electronics product
    let valid_book = json!({
        "name": "Rust Programming Book",
        "price": 49.99,
        "category": "books"
        // No warranty required for books
    });

    println!("   Valid book product (no warranty required):");
    println!("   Input JSON: {}", serde_json::to_string(&valid_book)?);

    let mut context = ValidationContext::from_json(&valid_book);
    match product_config.validate(&mut context) {
        Ok(_) => println!("   ✓ Book product validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Test invalid electronics product (missing warranty)
    let invalid_electronics = json!({
        "name": "Smartphone",
        "price": 699.99,
        "category": "electronics"
        // Missing warranty_months
    });

    println!("   Invalid electronics product (missing warranty):");
    println!(
        "   Input JSON: {}",
        serde_json::to_string(&invalid_electronics)?
    );

    let mut context = ValidationContext::from_json(&invalid_electronics);
    match product_config.validate(&mut context) {
        Ok(_) => println!("   ✗ Unexpected: Electronics without warranty passed validation"),
        Err(errors) => {
            println!("   ✓ Correctly failed: Electronics missing warranty");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Test JSON validation helper functions
    println!("\n🔧 Testing JSON validation helper functions:");

    // Test validate_json_str
    let json_str = r#"{"username": "testuser", "email": "test@example.com", "age": 30}"#;
    println!("   Testing JSON string validation:");
    println!("   Input: {}", json_str);

    match validate_json(json_str, &user_config) {
        Ok(_) => println!("   ✓ JSON string validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.path, error.message);
            }
        }
    }

    // Test validate_json_value
    let json_value = json!({
        "product": "Laptop",
        "specs": {
            "ram": "16GB",
            "storage": "512GB SSD"
        }
    });

    println!("   Testing JSON value validation:");
    println!("   Input: {}", serde_json::to_string(&json_value)?);

    match validate_json_value(&json_value, &product_config) {
        Ok(_) => println!("   ✓ JSON value validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.path, error.message);
            }
        }
    }

    println!();
    println!("🎉 JSON validation example completed!");
    println!("\n💡 Key Points:");
    println!("   • JSON validation works directly on serde_json::Value objects");
    println!("   • No deserialization to Rust structs required");
    println!("   • Supports nested JSON structures");
    println!("   • Dependencies work the same as with struct validation");
    println!("   • Extra JSON fields are ignored during validation");
    println!("   • Helper functions available for common JSON validation patterns");

    Ok(())
}
