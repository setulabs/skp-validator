//! Basic validation example demonstrating core validation features
//!
//! This example shows how to use the OneClick Validator for basic struct validation
//! with derive macros and manual configuration.

use oc_validator::prelude::*;
use serde::{Deserialize, Serialize};

/// User registration form with basic validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct UserRegistration {
    /// Username with length and format validation
    #[validate(
        required,
        length(min = 3, max = 20),
        regex(pattern = r"^[a-zA-Z0-9_]+$")
    )]
    pub username: String,

    /// Email with required and email format validation
    #[validate(required, email)]
    pub email: String,

    /// Password with length requirements
    #[validate(required, length(min = 8, max = 128))]
    pub password: String,

    /// Age with numeric range validation
    #[validate(range(min = 18, max = 120))]
    pub age: Option<u32>,

    /// Phone number with regex validation
    #[validate(regex(pattern = r"^\+?[1-9]\d{1,14}$"))]
    pub phone: Option<String>,

    /// Profile status with allowed values
    #[validate(allowed_values = ["active", "inactive", "pending"])]
    pub status: String,
}

/// Product with numeric and text validations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct Product {
    /// Product name with length constraints
    #[validate(required, length(min = 2, max = 100))]
    pub name: String,

    /// Price with numeric range
    #[validate(required, range(min = 0.01, max = 100000.0))]
    pub price: f64,

    /// Optional description with length limit
    #[validate(length(max = 500))]
    pub description: Option<String>,

    /// Stock quantity with integer range
    #[validate(int_range(min = 0, max = 10000))]
    pub stock_quantity: Option<i32>,

    /// Category with allowed values
    #[validate(allowed_values = ["electronics", "clothing", "books", "home", "sports"])]
    pub category: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 OneClick Validator - Basic Validation Example");
    println!("==============================================\n");

    // Test valid user registration
    println!("✅ Testing valid user registration:");
    let valid_user = UserRegistration {
        username: "john_doe123".to_string(),
        email: "john.doe@example.com".to_string(),
        password: "secure_password_123".to_string(),
        age: Some(25),
        phone: Some("+1234567890".to_string()),
        status: "active".to_string(),
    };

    match valid_user.validate() {
        Ok(_) => println!("   ✓ User registration is valid"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    println!();

    // Test invalid user registration
    println!("❌ Testing invalid user registration:");
    let invalid_user = UserRegistration {
        username: "u".to_string(),          // Too short
        email: "invalid-email".to_string(), // Invalid format
        password: "short".to_string(),      // Too short
        age: Some(150),                     // Too old
        phone: Some("invalid".to_string()), // Invalid format
        status: "banned".to_string(),       // Not in allowed values
    };

    match invalid_user.validate() {
        Ok(_) => println!("   ✗ Unexpected: Invalid user passed validation"),
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

    println!();

    // Test valid product
    println!("✅ Testing valid product:");
    let valid_product = Product {
        name: "Wireless Headphones".to_string(),
        price: 199.99,
        description: Some("High-quality wireless headphones with noise cancellation".to_string()),
        stock_quantity: Some(50),
        category: "electronics".to_string(),
    };

    match valid_product.validate() {
        Ok(_) => println!("   ✓ Product is valid"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    println!();

    // Test invalid product
    println!("❌ Testing invalid product:");
    let invalid_product = Product {
        name: "".to_string(),               // Empty name
        price: -10.0,                       // Negative price
        description: Some("A".repeat(600)), // Description too long
        stock_quantity: Some(-5),           // Negative stock
        category: "invalid".to_string(),    // Invalid category
    };

    match invalid_product.validate() {
        Ok(_) => println!("   ✗ Unexpected: Invalid product passed validation"),
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

    println!();
    println!("🎉 Basic validation example completed!");

    Ok(())
}
