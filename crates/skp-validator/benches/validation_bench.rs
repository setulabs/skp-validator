//! Validation performance benchmarks.

use std::hint::black_box;
use std::time::Instant;

use skp_validator::prelude::*;

fn main() {
    println!("SKP-Validator Performance Benchmarks\n");
    println!("{:=<60}", "");
    
    bench_string_validation();
    bench_struct_validation();
    bench_nested_validation();
    bench_collection_validation();
    
    println!("\n{:=<60}", "");
    println!("Benchmarks complete!");
}

fn bench_string_validation() {
    println!("\n📊 String Validation Benchmarks:");
    
    let email_rule = skp_validator::rules::EmailRule::new();
    let ctx = ValidationContext::default();
    
    let valid_email = "test@example.com";
    let invalid_email = "invalid";
    
    // Warm up
    for _ in 0..1000 {
        let _ = black_box(skp_validator_core::Rule::validate(&email_rule, valid_email, &ctx));
    }
    
    // Benchmark email validation
    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(skp_validator_core::Rule::validate(&email_rule, valid_email, &ctx));
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Email (valid):    {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(skp_validator_core::Rule::validate(&email_rule, invalid_email, &ctx));
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Email (invalid):  {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
    
    // Length rule
    let length_rule = skp_validator::rules::LengthRule::new().min(3).max(50);
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(skp_validator_core::Rule::validate(&length_rule, "hello world", &ctx));
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Length:           {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
    
    // Pattern rule
    let pattern_rule = skp_validator::rules::PatternRule::new(r"^\d{5}$");
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(skp_validator_core::Rule::validate(&pattern_rule, "12345", &ctx));
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Pattern (regex):  {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
}

#[derive(Debug, Clone)]
struct SimpleUser {
    name: String,
    email: String,
    age: u32,
}

impl Validate for SimpleUser {
    fn validate_with_context(&self, _ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();
        
        if self.name.trim().is_empty() {
            errors.add_field_error("name", 
                ValidationError::new("name", "required", "Name is required"));
        }
        
        if !self.email.contains('@') {
            errors.add_field_error("email",
                ValidationError::new("email", "email", "Invalid email"));
        }
        
        if self.age < 18 || self.age > 120 {
            errors.add_field_error("age",
                ValidationError::new("age", "range", "Age must be 18-120"));
        }
        
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn bench_struct_validation() {
    println!("\n📊 Struct Validation Benchmarks:");
    
    let valid_user = SimpleUser {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 25,
    };
    
    let invalid_user = SimpleUser {
        name: "".to_string(),
        email: "invalid".to_string(),
        age: 15,
    };
    
    let iterations = 100_000;
    
    // Valid struct
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(valid_user.validate());
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Simple struct (valid):    {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
    
    // Invalid struct
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(invalid_user.validate());
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Simple struct (invalid):  {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
}

#[derive(Debug, Clone)]
struct Address {
    street: String,
    city: String,
}

impl Validate for Address {
    fn validate_with_context(&self, _ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();
        if self.street.is_empty() {
            errors.add_field_error("street", ValidationError::new("street", "required", "Required"));
        }
        if self.city.is_empty() {
            errors.add_field_error("city", ValidationError::new("city", "required", "Required"));
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Clone)]
struct NestedUser {
    name: String,
    address: Address,
}

impl Validate for NestedUser {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();
        if self.name.is_empty() {
            errors.add_field_error("name", ValidationError::new("name", "required", "Required"));
        }
        if let Err(e) = self.address.validate_with_context(ctx) {
            errors.add_nested_errors("address", e);
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn bench_nested_validation() {
    println!("\n📊 Nested Validation Benchmarks:");
    
    let valid = NestedUser {
        name: "John".to_string(),
        address: Address { street: "123 Main".to_string(), city: "NYC".to_string() },
    };
    
    let iterations = 100_000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(valid.validate());
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Nested struct (2 levels): {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
}

#[derive(Debug, Clone)]
struct Tag {
    name: String,
}

impl Validate for Tag {
    fn validate_with_context(&self, _ctx: &ValidationContext) -> ValidationResult<()> {
        if self.name.len() < 2 {
            Err(ValidationErrors::from_iter([ValidationError::new("name", "length", "Too short")]))
        } else {
            Ok(())
        }
    }
}

fn bench_collection_validation() {
    println!("\n📊 Collection Validation Benchmarks:");
    
    let tags: Vec<Tag> = (0..10).map(|i| Tag { name: format!("tag{}", i) }).collect();
    let path = FieldPath::from_field("tags");
    let ctx = ValidationContext::default();
    
    let iterations = 50_000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = black_box(tags.validate_dive(&path, &ctx));
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
    println!("  Vec<Tag> (10 items):      {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
    
    // Larger collection
    let large_tags: Vec<Tag> = (0..100).map(|i| Tag { name: format!("tag{}", i) }).collect();
    let start = Instant::now();
    for _ in 0..(iterations / 10) {
        let _ = black_box(large_tags.validate_dive(&path, &ctx));
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / (iterations / 10) as f64;
    println!("  Vec<Tag> (100 items):     {:>8.2} ns/op ({:.2}M ops/sec)", 
             ns_per_op, 1_000.0 / ns_per_op);
}
