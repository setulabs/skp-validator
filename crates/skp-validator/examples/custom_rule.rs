use skp_validator::{Validate, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate)]
struct SignupRequest {
    #[validate(required, length(min = 6))]
    pub username: String,

    #[validate(required, length(min = 8), custom(function = "validate_password_complexity", message = "Password must maintain strict complexity"))]
    pub password: String,
}

fn validate_password_complexity(password: &str) -> Result<(), ValidationError> {
    // Requirements:
    // - At least one uppercase
    // - At least one lowercase
    // - At least one number
    // - At least one special char
    
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_number = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_upper && has_lower && has_number && has_special {
        Ok(())
    } else {
        Err(ValidationError::new(
            "password",
            "complexity",
            "Password must contain uppercase, lowercase, number, and special character",
        ))
    }
}

fn main() {
    let weak = SignupRequest {
        username: "user123".to_string(),
        password: "password123".to_string(), // Missing uppercase & special
    };

    println!("Validating weak password...");
    if let Err(e) = weak.validate() {
        for (field, errs) in e.field_errors() {
            println!("  - {}: {:?}", field, errs);
        }
    }

    let strong = SignupRequest {
        username: "user123".to_string(),
        password: "Password123!".to_string(),
    };

    println!("\nValidating strong password...");
    if strong.validate().is_ok() {
        println!("  Success!");
    }
}
