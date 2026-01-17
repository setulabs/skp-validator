//! Date and age validation example
//!
//! This example demonstrates date validation with custom formats and
//! age calculation from date of birth with min/max constraints.

use oc_validator::prelude::*;
use oc_validator::{
    FieldDependency, FieldValidationConfig, ValidationConfig, ValidationContext, ValidationRule,
};
use serde::{Deserialize, Serialize};

/// Person with date of birth and age validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct Person {
    /// Full name
    #[validate(required, length(min = 2, max = 100))]
    pub name: String,

    /// Date of birth with custom format
    #[validate(required, date(format = "%Y-%m-%d"))]
    pub date_of_birth: String,

    /// Age calculated from DOB with range validation
    #[validate(age(min = 18, max = 100, date_format = "%Y-%m-%d"))]
    pub age: Option<u32>,
}

/// Employee with various date validations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct Employee {
    /// Employee ID
    #[validate(required, length(min = 3, max = 10))]
    pub employee_id: String,

    /// Full name
    #[validate(required, length(min = 2, max = 100))]
    pub full_name: String,

    /// Date of birth
    #[validate(required, date(format = "%d-%m-%Y"))]
    pub date_of_birth: String,

    /// Age with different constraints
    #[validate(age(min = 21, max = 65, date_format = "%d-%m-%Y"))]
    pub age: Option<u32>,

    /// Joining date (must be after date of birth)
    #[validate(required, date(format = "%d-%m-%Y"))]
    pub joining_date: String,

    /// Contract end date (optional)
    #[validate(date(format = "%d-%m-%Y"))]
    pub contract_end_date: Option<String>,
}

/// Event with date range validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    pub title: String,
    pub start_date: String,
    pub end_date: String,
    pub max_attendees: Option<u32>,
}

impl Event {
    pub fn validation_config() -> ValidationConfig {
        let mut config = ValidationConfig::new();

        // Title validation
        config = config.add_field(
            FieldValidationConfig::new("title")
                .add_rule(ValidationRule::Required { message: None })
                .add_rule(ValidationRule::Length {
                    min: Some(3),
                    max: Some(200),
                    message: None,
                }),
        );

        // Start date validation
        config = config.add_field(
            FieldValidationConfig::new("start_date")
                .add_rule(ValidationRule::Required { message: None })
                .add_rule(ValidationRule::Date {
                    format: "%Y-%m-%d".to_string(),
                    message: Some("Start date must be in YYYY-MM-DD format".to_string()),
                }),
        );

        // End date validation
        config = config.add_field(
            FieldValidationConfig::new("end_date")
                .add_rule(ValidationRule::Required { message: None })
                .add_rule(ValidationRule::Date {
                    format: "%Y-%m-%d".to_string(),
                    message: Some("End date must be in YYYY-MM-DD format".to_string()),
                }),
        );

        // Date range dependency - end date must be after start date
        let date_range_dependency = FieldDependency::new(
            "end_date",
            "start_date",
            std::sync::Arc::new(|ctx| ctx.has_value("start_date") && ctx.has_value("end_date")),
        )
        .add_rule(ValidationRule::Contextual {
            validator: std::sync::Arc::new(|ctx| {
                use chrono::NaiveDate;

                let start_str = ctx.get_string("start_date").unwrap_or("");
                let end_str = ctx.get_string("end_date").unwrap_or("");

                let start_date =
                    NaiveDate::parse_from_str(start_str, "%Y-%m-%d").map_err(|_| {
                        vec![FieldError::new(
                            "end_date",
                            "Invalid start date format".to_string(),
                        )]
                    })?;

                let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%d").map_err(|_| {
                    vec![FieldError::new(
                        "end_date",
                        "Invalid end date format".to_string(),
                    )]
                })?;

                if end_date <= start_date {
                    return Err(vec![FieldError::new(
                        "end_date",
                        "End date must be after start date".to_string(),
                    )]);
                }

                Ok(())
            }),
            message: None,
        });

        config = config.add_dependency(date_range_dependency);

        // Max attendees validation
        config = config.add_field(FieldValidationConfig::new("max_attendees").add_rule(
            ValidationRule::Range {
                min: Some(1.0),
                max: Some(10000.0),
                message: Some("Max attendees must be between 1 and 10,000".to_string()),
            },
        ));

        config
    }

    pub fn validate_event(&self) -> Result<ValidationResult<()>, serde_json::Error> {
        let config = Self::validation_config();
        let mut context = ValidationContext::from_serde(self)?;
        Ok(config.validate(&mut context))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📅 OneClick Validator - Date and Age Validation Example");
    println!("===================================================\n");

    // Test person validation
    println!("👤 Testing person validation with DOB and age:");

    // Valid person
    println!("\n✅ Valid person (age 25):");
    let valid_person = Person {
        name: "John Doe".to_string(),
        date_of_birth: "1999-05-15".to_string(), // Should be ~25 years old
        age: Some(25),
    };

    match valid_person.validate() {
        Ok(_) => println!("   ✓ Person validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Invalid person - too young
    println!("\n❌ Invalid person (too young):");
    let young_person = Person {
        name: "Jane Smith".to_string(),
        date_of_birth: "2010-03-20".to_string(), // Would be ~14 years old
        age: Some(14),
    };

    match young_person.validate() {
        Ok(_) => println!("   ✗ Unexpected: Young person passed validation"),
        Err(errors) => {
            println!("   ✓ Correctly failed: Person too young");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Invalid person - invalid date format
    println!("\n❌ Invalid person (bad date format):");
    let bad_date_person = Person {
        name: "Bob Johnson".to_string(),
        date_of_birth: "1990/05/15".to_string(), // Wrong format
        age: Some(34),
    };

    match bad_date_person.validate() {
        Ok(_) => println!("   ✗ Unexpected: Invalid date format passed validation"),
        Err(errors) => {
            println!("   ✓ Correctly failed: Invalid date format");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Test employee validation
    println!("\n🏢 Testing employee validation:");

    // Valid employee
    println!("\n✅ Valid employee:");
    let valid_employee = Employee {
        employee_id: "EMP001".to_string(),
        full_name: "Alice Wilson".to_string(),
        date_of_birth: "25-12-1985".to_string(), // DD-MM-YYYY format
        age: Some(38),
        joining_date: "15-03-2020".to_string(),
        contract_end_date: Some("15-03-2025".to_string()),
    };

    match valid_employee.validate() {
        Ok(_) => println!("   ✓ Employee validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Invalid employee - too young for employment
    println!("\n❌ Invalid employee (too young):");
    let young_employee = Employee {
        employee_id: "EMP002".to_string(),
        full_name: "Charlie Brown".to_string(),
        date_of_birth: "10-05-2005".to_string(), // Would be ~19 years old
        age: Some(19),
        joining_date: "01-01-2024".to_string(),
        contract_end_date: None,
    };

    match young_employee.validate() {
        Ok(_) => println!("   ✗ Unexpected: Young employee passed validation"),
        Err(errors) => {
            println!("   ✓ Correctly failed: Employee too young (minimum 21)");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Test event validation with date ranges
    println!("\n📅 Testing event validation with date ranges:");

    // Valid event
    println!("\n✅ Valid event:");
    let valid_event = Event {
        title: "Rust Conference 2024".to_string(),
        start_date: "2024-09-15".to_string(),
        end_date: "2024-09-17".to_string(),
        max_attendees: Some(500),
    };

    match valid_event.validate_event() {
        Ok(Ok(_)) => println!("   ✓ Event validation passed"),
        Ok(Err(errors)) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Invalid event - end date before start date
    println!("\n❌ Invalid event (end before start):");
    let invalid_event = Event {
        title: "Invalid Conference".to_string(),
        start_date: "2024-09-20".to_string(),
        end_date: "2024-09-18".to_string(), // Before start date
        max_attendees: Some(200),
    };

    match invalid_event.validate_event() {
        Ok(Ok(_)) => println!("   ✗ Unexpected: Invalid date range passed validation"),
        Ok(Err(errors)) => {
            println!("   ✓ Correctly failed: End date before start date");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Invalid event - bad date format
    println!("\n❌ Invalid event (bad date format):");
    let bad_format_event = Event {
        title: "Conference".to_string(),
        start_date: "2024/09/15".to_string(), // Wrong format
        end_date: "2024/09/17".to_string(),   // Wrong format
        max_attendees: Some(100),
    };

    match bad_format_event.validate_event() {
        Ok(Ok(_)) => println!("   ✗ Unexpected: Bad date format passed validation"),
        Ok(Err(errors)) => {
            println!("   ✓ Correctly failed: Invalid date format");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
        Err(e) => println!("   ✗ JSON parsing error: {}", e),
    }

    // Test date range validation with JSON
    println!("\n📄 Testing date range validation with JSON:");

    let mut date_config = ValidationConfig::new();

    // Add date range validation
    date_config = date_config.add_field(FieldValidationConfig::new("event_date").add_rule(
        ValidationRule::DateRange {
            min: Some("2024-01-01".to_string()),
            max: Some("2024-12-31".to_string()),
            format: "%Y-%m-%d".to_string(),
            message: Some("Event date must be in 2024".to_string()),
        },
    ));

    // Valid date in range
    let valid_date_json = serde_json::json!({
        "event_date": "2024-06-15"
    });

    let mut context = ValidationContext::from_json(&valid_date_json);
    match date_config.validate(&mut context) {
        Ok(_) => println!("   ✓ Date in range validation passed"),
        Err(errors) => {
            println!("   ✗ Validation failed:");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    // Invalid date out of range
    let invalid_date_json = serde_json::json!({
        "event_date": "2025-01-01"  // Out of 2024 range
    });

    let mut context = ValidationContext::from_json(&invalid_date_json);
    match date_config.validate(&mut context) {
        Ok(_) => println!("   ✗ Unexpected: Date out of range passed validation"),
        Err(errors) => {
            println!("   ✓ Correctly failed: Date out of allowed range");
            for error in errors {
                println!("     - {}: {}", error.field, error.message);
            }
        }
    }

    println!();
    println!("🎉 Date and age validation example completed!");
    println!("\n💡 Key Points:");
    println!("   • Date validation supports custom formats (chrono strftime)");
    println!("   • Age validation calculates from DOB and supports min/max constraints");
    println!("   • Date ranges can validate against min/max date boundaries");
    println!("   • Dependencies can create complex date-based validation rules");
    println!("   • JSON validation supports the same date validation features");
    println!("   • All date operations are timezone-naive (NaiveDate)");

    Ok(())
}
