//! Example demonstrating struct validation with derive macro.

use skp_validator::prelude::*;

// In real usage: use skp_validator::Validate;
// For now, we'll implement Validate manually since the derive is being enhanced

/// A user registration form
#[derive(Debug)]
struct UserRegistration {
    username: String,
    email: String,
    password: String,
    password_confirm: String,
    age: u32,
}

impl Validate for UserRegistration {
    fn validate_with_context(&self, _ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        // Username: required, 3-20 characters
        if self.username.trim().is_empty() {
            errors.add_field_error("username", 
                ValidationError::new("username", "required", "Username is required"));
        } else if self.username.len() < 3 {
            errors.add_field_error("username",
                ValidationError::new("username", "length.min", "Username must be at least 3 characters"));
        } else if self.username.len() > 20 {
            errors.add_field_error("username",
                ValidationError::new("username", "length.max", "Username must be at most 20 characters"));
        }

        // Email: required, valid format
        if self.email.trim().is_empty() {
            errors.add_field_error("email",
                ValidationError::new("email", "required", "Email is required"));
        } else if !self.email.contains('@') || self.email.starts_with('@') || self.email.ends_with('@') {
            errors.add_field_error("email",
                ValidationError::new("email", "email", "Must be a valid email address"));
        }

        // Password: required, min 8 characters
        if self.password.is_empty() {
            errors.add_field_error("password",
                ValidationError::new("password", "required", "Password is required"));
        } else if self.password.len() < 8 {
            errors.add_field_error("password",
                ValidationError::new("password", "length.min", "Password must be at least 8 characters"));
        }

        // Password confirmation: must match password
        if self.password != self.password_confirm {
            errors.add_field_error("password_confirm",
                ValidationError::new("password_confirm", "must_match", "Passwords must match"));
        }

        // Age: 18-120
        if self.age < 18 {
            errors.add_field_error("age",
                ValidationError::new("age", "range.min", "Must be at least 18 years old"));
        } else if self.age > 120 {
            errors.add_field_error("age",
                ValidationError::new("age", "range.max", "Must be at most 120 years old"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn main() {
    println!("=== SKP Validator Example ===\n");

    // Valid registration
    let valid_user = UserRegistration {
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        password: "securepassword123".to_string(),
        password_confirm: "securepassword123".to_string(),
        age: 25,
    };

    println!("Testing valid user:");
    match valid_user.validate() {
        Ok(()) => println!("  ✓ Validation passed!\n"),
        Err(e) => println!("  ✗ Errors:\n{}\n", e),
    }

    // Invalid registration
    let invalid_user = UserRegistration {
        username: "ab".to_string(),  // Too short
        email: "invalid-email".to_string(),  // No @
        password: "short".to_string(),  // Too short
        password_confirm: "different".to_string(),  // Doesn't match
        age: 15,  // Too young
    };

    println!("Testing invalid user:");
    match invalid_user.validate() {
        Ok(()) => println!("  ✓ Validation passed!\n"),
        Err(errors) => {
            println!("  ✗ Found {} field(s) with errors:", errors.field_count());
            
            for (field, field_errors) in errors.field_errors() {
                println!("\n  Field '{}' errors:", field);
                match field_errors {
                    FieldErrors::Simple(errs) => {
                        for err in errs {
                            println!("    - [{}] {}", err.code, err.message);
                        }
                    }
                    _ => {}
                }
            }
            
            // Show JSON representation
            #[cfg(feature = "serde")]
            {
                println!("\n  JSON representation:");
                let json = serde_json::to_string_pretty(&errors).unwrap();
                for line in json.lines() {
                    println!("    {}", line);
                }
            }
        }
    }

    // Demonstrate programmatic validation with rules
    println!("\n=== Using Rules Directly ===\n");

    let email_rule = skp_validator::rules::EmailRule::new();
    let ctx = ValidationContext::default();

    let test_emails = ["valid@example.com", "invalid", "also@valid.org"];
    
    for email in &test_emails {
        match skp_validator_core::Rule::validate(&email_rule, *email, &ctx) {
            Ok(()) => println!("  '{}' - ✓ Valid email", email),
            Err(_) => println!("  '{}' - ✗ Invalid email", email),
        }
    }
}
