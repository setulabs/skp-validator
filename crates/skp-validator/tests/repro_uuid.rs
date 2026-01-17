use skp_validator::Validate;
use skp_validator_core::{ValidationContext, ValidationErrors};
use skp_validator::rules::UuidRule;
use skp_validator_core::Rule;

#[derive(Validate)]
struct TestStruct {
    #[validate(uuid)]
    pub id: String,
}

#[test]
fn test_repro_valid() {
    let s = TestStruct {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    };
    assert!(s.validate().is_ok());
}

#[test]
fn test_repro_invalid() {
    let s = TestStruct {
        id: "invalid-uuid".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let flat = errs.to_flat_map();
    println!("Errors: {:#?}", flat);
    assert!(flat.contains_key("id"));
}

#[test]
fn test_manual_rule() {
    let rule = UuidRule::new();
    let ctx = ValidationContext::default();
    let res = rule.validate("invalid-uuid", &ctx);
    assert!(res.is_err());
    
    let errs = res.unwrap_err();
    println!("Manual Rule Errors: {:#?}", errs);
    
    let mut final_errors = ValidationErrors::new();
    for err in errs.errors {
        final_errors.add_field_error("id", err);
    }
    
    let flat = final_errors.to_flat_map();
    println!("Manual Flat: {:#?}", flat);
    assert!(flat.contains_key("id"));
}
