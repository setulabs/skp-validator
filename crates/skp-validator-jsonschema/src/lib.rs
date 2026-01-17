//! JSON Schema generation for skp-validator.
//!
//! This crate provides utilities to generate JSON Schemas that include validation rules
//! defined using `skp-validator`. It integrates with `schemars`.

use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};
use schemars::JsonSchema;
use skp_validator_core::schema::{RuleSchema, TypeValidation, ValidationMetadata};

/// Generate a JSON Schema for a type, enriching it with validation rules.
pub fn schema_for<T: JsonSchema + ValidationMetadata>() -> Schema {
    let mut gen = SchemaGenerator::default();
    let mut schema = T::json_schema(&mut gen);
    
    let rules = T::get_validation_rules();
    enrich_schema(&mut schema, &rules);
    
    schema
}

fn enrich_schema(schema: &mut Schema, rules: &TypeValidation) {
    if let Schema::Object(schema_obj) = schema {
        // Handle fields if this is an object
        if let Some(object) = &mut schema_obj.object {
            for (prop_name, prop_schema) in &mut object.properties {
                if let Some(field_rules) = rules.fields.get(prop_name) {
                    enrich_field_schema(prop_schema, &field_rules.rules);
                }
                
                // Recursively enrich nested types
                if let Some(nested_rules) = rules.nested.get(prop_name) {
                    enrich_nested(prop_schema, nested_rules);
                }
            }
        }
    }
}

fn enrich_nested(schema: &mut Schema, rules: &TypeValidation) {
    if let Schema::Object(obj) = schema {
        if let Some(array) = &mut obj.array {
            // If it's an array, applying nested rules means applying them to the items
            if let Some(SingleOrVec::Single(item_schema)) = &mut array.items {
                enrich_schema(item_schema, rules);
            }
        } else {
            // Otherwise apply to the object itself
            enrich_schema(schema, rules);
        }
    }
}

fn enrich_field_schema(schema: &mut Schema, rules: &[RuleSchema]) {
    if let Schema::Object(schema_obj) = schema {
        for rule in rules {
            apply_rule(schema_obj, rule);
        }
    }
}

fn apply_rule(schema: &mut SchemaObject, rule: &RuleSchema) {
    match rule {
        RuleSchema::Length { min, max, .. } => {
            if let Some(min) = min {
                schema_ensure_string(schema);
                schema.string().min_length = Some(*min as u32);
            }
            if let Some(max) = max {
                schema_ensure_string(schema);
                schema.string().max_length = Some(*max as u32);
            }
        }
        RuleSchema::Range { min, max, .. } => {
             if let Some(min) = min {
                 schema_ensure_number(schema);
                 schema.number().minimum = Some(*min);
             }
             if let Some(max) = max {
                 schema_ensure_number(schema);
                 schema.number().maximum = Some(*max);
             }
        }
        RuleSchema::Pattern { regex } => {
            schema_ensure_string(schema);
            schema.string().pattern = Some(regex.clone());
        }
        RuleSchema::Email => {
            schema_ensure_string(schema);
            schema.format = Some("email".to_string());
        }
        RuleSchema::Url => {
            schema_ensure_string(schema);
            schema.format = Some("uri".to_string());
        }
        RuleSchema::Ip { .. } => {
            schema_ensure_string(schema);
            schema.format = Some("ip".to_string()); // or ipv4/ipv6
        }
        RuleSchema::Uuid { .. } => {
            schema_ensure_string(schema);
            schema.format = Some("uuid".to_string());
        }
        _ => {}
    }
}

fn schema_ensure_string(schema: &mut SchemaObject) {
    // Basic check to ensure we are modifying a string schema or adding string constraint
    // schemars usually sets the type.
    if schema.instance_type.is_none() {
        schema.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::String)));
    }
}

fn schema_ensure_number(schema: &mut SchemaObject) {
    if schema.instance_type.is_none() {
        schema.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::Number)));
    }
}

trait SchemaHelpers {
    fn string(&mut self) -> &mut schemars::schema::StringValidation;
    fn number(&mut self) -> &mut schemars::schema::NumberValidation;
}

impl SchemaHelpers for SchemaObject {
    fn string(&mut self) -> &mut schemars::schema::StringValidation {
        self.string.get_or_insert_with(Default::default)
    }
    fn number(&mut self) -> &mut schemars::schema::NumberValidation {
        self.number.get_or_insert_with(Default::default)
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
        
        if let Schema::Object(obj) = schema {
            let props = obj.object.unwrap().properties;
            
            // Check name field
            let name_schema = props.get("name").unwrap();
            if let Schema::Object(name_obj) = name_schema {
                let string_validation = name_obj.string.as_ref().unwrap();
                assert_eq!(string_validation.min_length, Some(5));
            } else {
                panic!("name schema is not an object");
            }
            
            // Check age field
            let age_schema = props.get("age").unwrap();
            if let Schema::Object(age_obj) = age_schema {
                let number_validation = age_obj.number.as_ref().unwrap();
                assert_eq!(number_validation.minimum, Some(18.0));
                assert_eq!(number_validation.maximum, Some(100.0));
            } else {
                panic!("age schema is not an object");
            }
        } else {
            panic!("Schema is not an object");
        }
    }
}
