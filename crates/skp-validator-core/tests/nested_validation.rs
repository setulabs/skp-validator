//! Tests for nested and collection validation.

use skp_validator_core::prelude::*;
use std::collections::{HashMap, BTreeMap};

// === Test types ===

#[derive(Debug, Clone)]
struct Address {
    street: String,
    city: String,
}

impl Validate for Address {
    fn validate_with_context(&self, _ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        if self.street.trim().is_empty() {
            errors.add_field_error("street", 
                ValidationError::new("street", "required", "Street is required"));
        }

        if self.city.trim().is_empty() {
            errors.add_field_error("city",
                ValidationError::new("city", "required", "City is required"));
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Clone)]
struct Tag {
    name: String,
}

impl Validate for Tag {
    fn validate_with_context(&self, _ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        if self.name.len() < 2 {
            errors.add_field_error("name",
                ValidationError::new("name", "length.min", "Tag name must be at least 2 characters"));
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug)]
struct User {
    name: String,
    address: Address,
    tags: Vec<Tag>,
    metadata: HashMap<String, Tag>,
}

impl Validate for User {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        // Validate name
        if self.name.trim().is_empty() {
            errors.add_field_error("name",
                ValidationError::new("name", "required", "Name is required"));
        }

        // Validate nested address
        if let Err(nested_errors) = self.address.validate_with_context(ctx) {
            errors.add_nested_errors("address", nested_errors);
        }

        // Validate tag collection (dive)
        let path = FieldPath::from_field("tags");
        if let Err(tag_errors) = self.tags.validate_dive(&path, ctx) {
            errors.merge(tag_errors);
        }

        // Validate metadata map (dive)
        let meta_path = FieldPath::from_field("metadata");
        if let Err(meta_errors) = self.metadata.validate_dive(&meta_path, ctx) {
            errors.merge(meta_errors);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// === Tests ===

#[test]
fn test_nested_validation_passes() {
    let user = User {
        name: "John".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
        },
        tags: vec![
            Tag { name: "developer".to_string() },
            Tag { name: "rust".to_string() },
        ],
        metadata: HashMap::new(),
    };

    assert!(user.validate().is_ok());
}

#[test]
fn test_nested_validation_fails() {
    let user = User {
        name: "John".to_string(),
        address: Address {
            street: "".to_string(),  // Invalid
            city: "New York".to_string(),
        },
        tags: vec![],
        metadata: HashMap::new(),
    };

    let result = user.validate();
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    assert!(errors.has_field_error("address"));
    
    // Check nested error
    let flat = errors.to_flat_map();
    assert!(flat.contains_key("address.street"));
}

#[test]
fn test_vec_dive_validation() {
    let user = User {
        name: "John".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
        },
        tags: vec![
            Tag { name: "ok".to_string() },
            Tag { name: "x".to_string() },  // Too short
            Tag { name: "valid".to_string() },
        ],
        metadata: HashMap::new(),
    };

    let result = user.validate();
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    assert!(errors.has_field_error("tags"));
    
    // Check the error is at index 1
    if let Some(FieldErrors::List(list)) = errors.field("tags") {
        assert!(list.contains_key(&1));
        assert!(!list.contains_key(&0));
        assert!(!list.contains_key(&2));
    } else {
        panic!("Expected list errors for tags");
    }
}

#[test]
fn test_hashmap_dive_validation() {
    let mut metadata = HashMap::new();
    metadata.insert("valid".to_string(), Tag { name: "good".to_string() });
    metadata.insert("invalid".to_string(), Tag { name: "x".to_string() }); // Too short

    let user = User {
        name: "John".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
        },
        tags: vec![],
        metadata,
    };

    let result = user.validate();
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    assert!(errors.has_field_error("metadata"));
    
    // Check the error is at key "invalid"
    if let Some(FieldErrors::Map(map)) = errors.field("metadata") {
        assert!(map.contains_key("invalid"));
        assert!(!map.contains_key("valid"));
    } else {
        panic!("Expected map errors for metadata");
    }
}

#[test]
fn test_btreemap_dive() {
    #[derive(Debug)]
    struct Container {
        items: BTreeMap<String, Tag>,
    }

    impl Validate for Container {
        fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
            let mut errors = ValidationErrors::new();
            let path = FieldPath::from_field("items");
            if let Err(e) = self.items.validate_dive(&path, ctx) {
                errors.merge(e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }

    let mut items = BTreeMap::new();
    items.insert("a".to_string(), Tag { name: "good".to_string() });
    items.insert("b".to_string(), Tag { name: "x".to_string() }); // Invalid

    let container = Container { items };
    let result = container.validate();
    
    assert!(result.is_err());
    let errors = result.unwrap_err();
    
    if let Some(FieldErrors::Map(map)) = errors.field("items") {
        assert!(map.contains_key("b"));
        assert!(!map.contains_key("a"));
    }
}

#[test]
fn test_option_dive() {
    #[derive(Debug)]
    struct Container {
        maybe_address: Option<Address>,
    }

    impl Validate for Container {
        fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
            let mut errors = ValidationErrors::new();
            let path = FieldPath::from_field("maybe_address");
            if let Err(e) = self.maybe_address.validate_dive(&path, ctx) {
                errors.merge(e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }

    // None should pass
    let container = Container { maybe_address: None };
    assert!(container.validate().is_ok());

    // Some with valid address should pass
    let container = Container {
        maybe_address: Some(Address {
            street: "123 Main".to_string(),
            city: "NYC".to_string(),
        }),
    };
    assert!(container.validate().is_ok());

    // Some with invalid address should fail
    let container = Container {
        maybe_address: Some(Address {
            street: "".to_string(),
            city: "NYC".to_string(),
        }),
    };
    let result = container.validate();
    assert!(result.is_err());
}

#[test]
fn test_box_dive() {
    #[derive(Debug)]
    struct Container {
        boxed: Box<Address>,
    }

    impl Validate for Container {
        fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
            let mut errors = ValidationErrors::new();
            let path = FieldPath::from_field("boxed");
            if let Err(e) = self.boxed.validate_dive(&path, ctx) {
                errors.merge(e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }

    // Valid
    let container = Container {
        boxed: Box::new(Address {
            street: "123 Main".to_string(),
            city: "NYC".to_string(),
        }),
    };
    assert!(container.validate().is_ok());

    // Invalid
    let container = Container {
        boxed: Box::new(Address {
            street: "".to_string(),
            city: "".to_string(),
        }),
    };
    assert!(container.validate().is_err());
}

#[test]
fn test_array_dive() {
    #[derive(Debug)]
    struct Container {
        fixed: [Tag; 3],
    }

    impl Validate for Container {
        fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
            let mut errors = ValidationErrors::new();
            let path = FieldPath::from_field("fixed");
            if let Err(e) = self.fixed.validate_dive(&path, ctx) {
                errors.merge(e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }

    // All valid
    let container = Container {
        fixed: [
            Tag { name: "aa".to_string() },
            Tag { name: "bb".to_string() },
            Tag { name: "cc".to_string() },
        ],
    };
    assert!(container.validate().is_ok());

    // One invalid
    let container = Container {
        fixed: [
            Tag { name: "aa".to_string() },
            Tag { name: "x".to_string() },  // Too short
            Tag { name: "cc".to_string() },
        ],
    };
    let result = container.validate();
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    if let Some(FieldErrors::List(list)) = errors.field("fixed") {
        assert!(list.contains_key(&1));
        assert_eq!(list.len(), 1);
    }
}

#[test]
fn test_deeply_nested() {
    #[derive(Debug)]
    struct Level2 {
        value: String,
    }

    impl Validate for Level2 {
        fn validate_with_context(&self, _ctx: &ValidationContext) -> ValidationResult<()> {
            if self.value.is_empty() {
                Err(ValidationErrors::from_iter([
                    ValidationError::new("value", "required", "Value is required")
                ]))
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug)]
    struct Level1 {
        nested: Level2,
    }

    impl Validate for Level1 {
        fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
            let mut errors = ValidationErrors::new();
            if let Err(e) = self.nested.validate_with_context(ctx) {
                errors.add_nested_errors("nested", e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }

    #[derive(Debug)]
    struct Root {
        level1: Level1,
    }

    impl Validate for Root {
        fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
            let mut errors = ValidationErrors::new();
            if let Err(e) = self.level1.validate_with_context(ctx) {
                errors.add_nested_errors("level1", e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }

    let root = Root {
        level1: Level1 {
            nested: Level2 {
                value: "".to_string(), // Invalid
            },
        },
    };

    let result = root.validate();
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    let flat = errors.to_flat_map();
    
    // Should have deeply nested path
    assert!(flat.contains_key("level1.nested.value"));
}

#[test]
fn test_error_count_nested() {
    let user = User {
        name: "".to_string(),  // Error 1
        address: Address {
            street: "".to_string(),  // Error 2
            city: "".to_string(),    // Error 3
        },
        tags: vec![
            Tag { name: "x".to_string() },  // Error 4
            Tag { name: "y".to_string() },  // Error 5
        ],
        metadata: HashMap::new(),
    };

    let result = user.validate();
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    assert_eq!(errors.count(), 5);
}
