use skp_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq)]
pub struct ECommerceOrder {
    #[validate(uuid)]
    pub id: String,

    #[validate(custom(function = "validate_date", message = "Invalid date format"))]
    pub date: String,

    #[validate(nested)]
    pub user: UserProfile,

    #[validate(dive)]
    pub items: Vec<OrderItem>,

    #[validate(nested)]
    pub shipping_address: Address,

    #[validate(nested)]
    pub billing_address: Address,
    
    #[validate(range(min = 0.0, message = "Total must be positive"))]
    pub total: f64,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    #[validate(length(min = 3, max = 50, message = "Name must be 3-50 chars"))]
    pub name: String,
    
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    
    #[validate(phone(message = "Invalid phone"))]
    pub phone: String,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq)]
pub struct OrderItem {
    #[validate(uuid(message = "Invalid product ID"))]
    pub product_id: String,
    
    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: u32,
    
    #[validate(range(min = 0.01, message = "Price must be positive"))]
    pub price: f64,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq)]
pub struct Address {
    #[validate(length(min = 5, message = "Street address too short"))]
    pub street: String,
    
    #[validate(length(min = 2, message = "City name too short"))]
    pub city: String,
    
    // Simple zip regex
    #[validate(pattern(regex = "^\\d{5}(-\\d{4})?$", message = "Does not match the required format"))]
    pub zip: String,
    
    #[validate(allowed_values(values = ["US", "CA", "UK"], message = "Invalid country"))]
    pub country: String,
}

fn validate_date(date: &str) -> Result<(), skp_validator::ValidationError> {
    // Simple ISO date check YYYY-MM-DD
    if date.len() == 10 && date.chars().nth(4) == Some('-') && date.chars().nth(7) == Some('-') {
        Ok(())
    } else {
        Err(skp_validator::ValidationError::new("date", "date_format", "Invalid date format"))
    }
}

#[test]
fn test_valid_complex_order() {
    let order = ECommerceOrder {
        id: "123e4567-e89b-12d3-a456-426614174000".to_string(), // Valid UUID
        date: "2023-10-27".to_string(),
        user: UserProfile {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            phone: "+16502530000".to_string(), // Google US
        },
        items: vec![
            OrderItem {
                product_id: "123e4567-e89b-12d3-a456-426614174001".to_string(),
                quantity: 2,
                price: 19.99,
            },
            OrderItem {
                product_id: "123e4567-e89b-12d3-a456-426614174002".to_string(),
                quantity: 1,
                price: 5.00,
            }
        ],
        shipping_address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
            country: "US".to_string(),
        },
        billing_address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
            country: "US".to_string(),
        },
        total: 44.98,
    };

    let result = order.validate();
    if let Err(e) = &result {
        println!("Validation failed: {}", e);
        // Print detailed
        for (field, errs) in e.field_errors() {
             println!("{}: {:?}", field, errs);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_mixed_failures_deeply_nested() {
    let order = ECommerceOrder {
         id: "invalid-uuid".to_string(),
         date: "2023/10/27".to_string(), // Invalid format
         user: UserProfile {
             name: "Jo".to_string(), // Too short
             email: "invalid-email".to_string(),
             phone: "123".to_string(), // Invalid phone
         },
         items: vec![
             OrderItem {
                 product_id: "bad-id".to_string(),
                 quantity: 0, // Invalid
                 price: -1.0, // Invalid
             }
         ],
         shipping_address: Address {
             street: "Tiny".to_string(), // Length 4, min 5 -> Invalid
             city: "NY".to_string(),
             zip: "ABCDE".to_string(), // Invalid pattern
             country: "FR".to_string(), // Invalid country
         },
         billing_address: Address {
             street: "123 Correct St".to_string(),
             city: "London".to_string(),
             zip: "12345".to_string(),
             country: "UK".to_string(),
         },
         total: -100.0, // Invalid
    };

    let result = order.validate();
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    let flat_errors = errors.to_flat_map();
    
    println!("Validation Errors: {:#?}", flat_errors);
    
    // Top level errors
    assert!(flat_errors.contains_key("id")); 
    assert_eq!(flat_errors["id"], vec!["Invalid UUID"]);
    assert!(flat_errors.contains_key("date"));
    assert_eq!(flat_errors["date"], vec!["Invalid date format"]);
    
    // Total validation check (range)
    assert!(flat_errors.contains_key("total"));
    
    // User errors
    assert!(flat_errors.contains_key("user.name"));
    assert!(flat_errors.contains_key("user.email"));
    assert!(flat_errors.contains_key("user.phone")); 
    
    // Item errors
    assert!(flat_errors.contains_key("items[0].product_id"));
    assert!(flat_errors.contains_key("items[0].quantity"));
    assert!(flat_errors.contains_key("items[0].price"));
    
    // Shipping Address
    assert!(flat_errors.contains_key("shipping_address.street")); 
    assert!(flat_errors.contains_key("shipping_address.zip")); 
    assert!(flat_errors.contains_key("shipping_address.country"));
    
    // Billing address should be valid (no errors)
    assert!(!flat_errors.contains_key("billing_address.zip"));
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq)]
pub struct RecursiveNode {
    #[validate(length(min = 1, message = "Name required"))]
    pub name: String,
    
    #[validate(dive)]
    pub children: Vec<RecursiveNode>,
}

#[test]
fn test_recursive_validation() {
    let node = RecursiveNode {
        name: "root".to_string(),
        children: vec![
             RecursiveNode {
                 name: "child1".to_string(),
                 children: vec![],
             },
             RecursiveNode {
                 name: "".to_string(), // Invalid
                 children: vec![
                     RecursiveNode {
                         name: "subchild".to_string(),
                         children: vec![],
                     }
                 ],
             }
        ]
    };
    
    let result = node.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let flat = errors.to_flat_map();
    println!("Recursive errors: {:#?}", flat);
    
    assert!(flat.contains_key("children[1].name"));
    assert_eq!(flat["children[1].name"], vec!["Name required"]);
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq)]
pub struct ConfigMap {
    #[validate(length(min = 3))]
    pub env: String,
    
    #[validate(dive)]
    pub settings: std::collections::HashMap<String, ConfigItem>,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize, PartialEq)]
pub struct ConfigItem {
    #[validate(range(min = 0, message = "Value must be positive"))]
    pub value: i32,
}

#[test]
fn test_hashmap_validation() {
    use std::collections::HashMap;
    let mut settings = HashMap::new();
    settings.insert("timeout".to_string(), ConfigItem { value: 30 });
    settings.insert("retry".to_string(), ConfigItem { value: -1 }); // Invalid
    
    let config = ConfigMap {
        env: "prod".to_string(),
        settings,
    };
    
    let result = config.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let flat = errors.to_flat_map();
    println!("Map errors: {:#?}", flat);
    
    assert!(flat.contains_key("settings[retry].value"));
    assert_eq!(flat["settings[retry].value"], vec!["Value must be positive"]);
}
