use skp_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate)]
struct Company {
    #[validate(required, length(min = 3))]
    pub name: String,

    #[validate(dive)]
    pub departments: Vec<Department>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct Department {
    #[validate(required, length(min = 2))]
    pub name: String,

    #[validate(dive)]
    pub employees: Vec<Employee>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct Employee {
    #[validate(required)]
    pub name: String,

    #[validate(email)]
    pub email: String,
    
    #[validate(phone)]
    pub phone: Option<String>,
}

fn main() {
    let company = Company {
        name: "Tech Corp".to_string(),
        departments: vec![
            Department {
                name: "Engineering".to_string(),
                employees: vec![
                    Employee {
                        name: "Alice".to_string(),
                        email: "alice@tech.corp".to_string(),
                        phone: None,
                    },
                    Employee {
                        name: "Bob".to_string(),
                        email: "bob@tech.corp".to_string(),
                        phone: Some("+15550199".to_string()),
                    },
                ],
            },
            Department {
                name: "HR".to_string(),
                employees: vec![
                    Employee {
                        name: "".to_string(), // Invalid
                        email: "invalid-email".to_string(), // Invalid
                        phone: None,
                    }
                ],
            }
        ],
    };

    println!("Validating company structure...");
    match company.validate() {
        Ok(_) => println!("Valid!"),
        Err(e) => {
            println!("Validation failed with {} errors:", e.errors.len() + e.fields.len()); // simplistic count
            // Flatten generic error map makes it easy to see where issues are
            for (path, errors) in e.to_flat_map() {
                 println!("- {}: {:?}", path, errors);
            }
        }
    }
}
