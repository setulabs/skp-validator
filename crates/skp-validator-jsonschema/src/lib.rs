//! JSON Schema generation for skp-validator.
//!
//! This crate provides utilities to generate JSON Schemas that include validation rules
//! defined using `skp-validator`. It integrates with `schemars`.

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde_json::json;
use skp_validator_core::schema::{RuleSchema, TypeValidation, ValidationMetadata};

/// Generate a JSON Schema for a type, enriching it with validation rules.
pub fn schema_for<T: JsonSchema + ValidationMetadata>() -> Schema {
    let mut schema_gen = SchemaGenerator::default();
    let mut schema = T::json_schema(&mut schema_gen);
    
    let rules = T::get_validation_rules();
    if let Some(schema_v) = schema.pointer_mut("") {
        enrich_schema_value(schema_v, &rules);
    }
    
    schema
}

fn enrich_schema_value(schema_v: &mut serde_json::Value, rules: &TypeValidation) {
    if let Some(obj) = schema_v.as_object_mut() {
        // Handle fields if this is an object
        if let Some(properties) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
            for (prop_name, prop_schema_v) in properties {
                if let Some(field_rules) = rules.fields.get(prop_name) {
                    enrich_field_value(prop_schema_v, &field_rules.rules);
                }
                
                // Recursively enrich nested types
                if let Some(nested_rules) = rules.nested.get(prop_name) {
                    enrich_nested_value(prop_schema_v, nested_rules);
                }
            }
        }
    }
}

fn enrich_nested_value(schema_v: &mut serde_json::Value, rules: &TypeValidation) {
    if let Some(items) = schema_v.get_mut("items") {
        // If it's an array, applying nested rules means applying them to the items
        enrich_schema_value(items, rules);
    } else {
        // Otherwise apply to the object itself
        enrich_schema_value(schema_v, rules);
    }
}

fn enrich_field_value(schema_v: &mut serde_json::Value, rules: &[RuleSchema]) {
    for rule in rules {
        apply_rule_value(schema_v, rule);
    }
}

fn apply_rule_value(schema_v: &mut serde_json::Value, rule: &RuleSchema) {
    match rule {
        RuleSchema::Length { min, max, .. } => {
            if let Some(min) = min {
                ensure_type(schema_v, "string");
                schema_v["minLength"] = json!(min);
            }
            if let Some(max) = max {
                ensure_type(schema_v, "string");
                schema_v["maxLength"] = json!(max);
            }
        }
        RuleSchema::Range { min, max, .. } => {
             if let Some(min) = min {
                 ensure_type(schema_v, "number");
                 schema_v["minimum"] = json!(min);
             }
             if let Some(max) = max {
                 ensure_type(schema_v, "number");
                 schema_v["maximum"] = json!(max);
             }
        }
        RuleSchema::Pattern { regex } => {
            ensure_type(schema_v, "string");
            schema_v["pattern"] = json!(regex);
        }
        RuleSchema::Email => {
            ensure_type(schema_v, "string");
            schema_v["format"] = json!("email");
        }
        RuleSchema::Url => {
            ensure_type(schema_v, "string");
            schema_v["format"] = json!("uri");
        }
        RuleSchema::Ip { .. } => {
            ensure_type(schema_v, "string");
            schema_v["format"] = json!("ip");
        }
        RuleSchema::Uuid { .. } => {
            ensure_type(schema_v, "string");
            schema_v["format"] = json!("uuid");
        }
        _ => {}
    }
}

fn ensure_type(schema_v: &mut serde_json::Value, expected_type: &str) {
    if let Some(existing_type) = schema_v.get_mut("type") {
        if existing_type.is_string() {
            // Already a string, if it's not the expected type we might have a conflict
            // but for now we follow the metadata's lead.
        } else if let Some(arr) = existing_type.as_array_mut()
            && !arr.iter().any(|v| v.as_str() == Some(expected_type)) {
                arr.push(json!(expected_type));
        }
    } else {
        schema_v["type"] = json!(expected_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use skp_validator::Validate;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
    struct TestStruct {
        #[validate(length(min = 5))]
        name: String,
        #[validate(range(min = 18.0, max = 100.0))]
        age: u32,
    }

    #[test]
    fn test_schema_generation() {
        let schema = schema_for::<TestStruct>();
        let json = serde_json::to_string_pretty(&schema).unwrap();
        println!("{}", json);
        
        let schema_v = schema.as_value();
        let properties = schema_v["properties"].as_object().expect("Schema should have properties");
        
        // Check name field
        let name_schema = &properties["name"];
        assert_eq!(name_schema["minLength"], json!(5));
        
        // Check age field
        let age_schema = &properties["age"];
        assert_eq!(age_schema["minimum"], json!(18.0));
        assert_eq!(age_schema["maximum"], json!(100.0));
    }
}
