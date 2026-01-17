//! Field transformations example
//!
//! This example demonstrates how to use field transformations (uppercase, lowercase, trim)
//! during validation. Transformations are applied before validation rules are checked.

use oc_validator::prelude::*;
use serde::{Deserialize, Serialize};

/// User profile with field transformations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct UserProfile {
    /// Username is transformed to uppercase before validation
    #[validate(
        required,
        length(min = 3, max = 15),
        regex(pattern = r"^[A-Z0-9_]+$"),
        uppercase
    )]
    pub username: String,

    /// Email is transformed to lowercase before validation
    #[validate(required, email, lowercase)]
    pub email: String,

    /// Full name with trimming
    #[validate(required, length(min = 2, max = 100), trim)]
    pub full_name: String,

    /// Bio with trimming and length validation
    #[validate(length(max = 500), trim)]
    pub bio: Option<String>,

    /// Profile type with uppercase transformation and allowed values
    #[validate(required, allowed_values = ["BASIC", "PREMIUM", "VIP"], uppercase)]
    pub profile_type: String,
}

/// Address with comprehensive transformations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct Address {
    /// Street address with trimming
    #[validate(required, length(min = 5, max = 200), trim)]
    pub street: String,

    /// City with uppercase transformation
    #[validate(required, length(min = 2, max = 100), uppercase)]
    pub city: String,

    /// State with uppercase transformation
    #[validate(required, length(min = 2, max = 50), uppercase)]
    pub state: String,

    /// Country with uppercase transformation and allowed values
    #[validate(required, allowed_values = ["USA", "CANADA", "UK", "AUSTRALIA", "GERMANY"], uppercase)]
    pub country: String,

    /// Postal code with trimming and format validation
    #[validate(required, regex(pattern = r"^\d{5}(-\d{4})?$"), trim)]
    pub postal_code: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 OneClick Validator - Field Transformations Example");
    println!("==================================================\n");

    // Test user profile transformations
    println!("👤 Testing user profile transformations:");
    let profile = UserProfile {
        username: "john_doe".to_string(),          // Will become "JOHN_DOE"
        email: "John.Doe@Example.COM".to_string(), // Will become "john.doe@example.com"
        full_name: "  john doe  ".to_string(),     // Will become "john doe"
        bio: Some("  Hello, I'm a developer!  ".to_string()), // Will become "Hello, I'm a developer!"
        profile_type: "premium".to_string(),                  // Will become "PREMIUM"
    };

    println!("Before validation:");
    println!("  Username: '{}'", profile.username);
    println!("  Email: '{}'", profile.email);
    println!("  Full Name: '{:?}'", profile.full_name);
    println!("  Bio: '{:?}'", profile.bio);
    println!("  Profile Type: '{}'", profile.profile_type);

    match profile.validate() {
        Ok(_) => {
            println!("\n✅ Validation passed! Transformations applied:");
            println!("  Username: '{}'", profile.username);
            println!("  Email: '{}'", profile.email);
            println!("  Full Name: '{:?}'", profile.full_name);
            println!("  Bio: '{:?}'", profile.bio);
            println!("  Profile Type: '{}'", profile.profile_type);
        }
        Err(errors) => {
            println!("\n❌ Validation failed:");
            for error in errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    println!();

    // Test address transformations
    println!("🏠 Testing address transformations:");
    let address = Address {
        street: "  123 Main Street  ".to_string(), // Will be trimmed
        city: "new york".to_string(),              // Will become "NEW YORK"
        state: "ny".to_string(),                   // Will become "NY"
        country: "usa".to_string(),                // Will become "USA"
        postal_code: "  10001  ".to_string(),      // Will be trimmed
    };

    println!("Before validation:");
    println!("  Street: '{}'", address.street);
    println!("  City: '{}'", address.city);
    println!("  State: '{}'", address.state);
    println!("  Country: '{}'", address.country);
    println!("  Postal Code: '{}'", address.postal_code);

    match address.validate() {
        Ok(_) => {
            println!("\n✅ Validation passed! Transformations applied:");
            println!("  Street: '{}'", address.street);
            println!("  City: '{}'", address.city);
            println!("  State: '{}'", address.state);
            println!("  Country: '{}'", address.country);
            println!("  Postal Code: '{}'", address.postal_code);
        }
        Err(errors) => {
            println!("\n❌ Validation failed:");
            for error in errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    println!();

    // Test invalid transformations (should fail validation)
    println!("❌ Testing invalid data with transformations:");
    let invalid_profile = UserProfile {
        username: "u".to_string(),          // Too short even after uppercase
        email: "invalid-email".to_string(), // Invalid format even after lowercase
        full_name: "  x  ".to_string(),     // Too short even after trim
        bio: Some("A".repeat(600)),         // Too long even after trim
        profile_type: "gold".to_string(),   // Not in allowed values even after uppercase
    };

    match invalid_profile.validate() {
        Ok(_) => println!("   ✗ Unexpected: Invalid profile passed validation"),
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
    println!("🎉 Field transformations example completed!");
    println!("\n💡 Key Points:");
    println!("   • Transformations are applied BEFORE validation rules");
    println!("   • This allows transformed values to pass validation");
    println!("   • Original input data is modified during validation");
    println!("   • Use transformations for data normalization");

    Ok(())
}
