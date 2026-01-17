use skp_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate)]
struct User {
    #[validate(required, length(min = 3, max = 50))]
    pub name: String,

    #[validate(required, email)]
    pub email: String,

    #[validate(range(min = 18, max = 120))]
    pub age: Option<u32>,

    #[validate(url)]
    pub website: Option<String>,
}

fn main() {
    let user = User {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: Some(25),
        website: Some("https://example.com".to_string()),
    };

    match user.validate() {
        Ok(_) => println!("User is valid!"),
        Err(e) => println!("Validation errors: {:?}", e),
    }

    let invalid_user = User {
        name: "Al".to_string(), // Too short
        email: "invalid-email".to_string(), // Invalid email
        age: Some(15), // Too young
        website: Some("not-a-url".to_string()), // Invalid URL
    };

    match invalid_user.validate() {
        Ok(_) => println!("Invalid user is somehow valid?"),
        Err(e) => {
            println!("\nExpected validation errors:");
            for (field, errors) in e.field_errors() {
                println!("{}: {:?}", field, errors);
            }
        }
    }
}
