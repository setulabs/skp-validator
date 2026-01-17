use actix_web::{test, App, HttpResponse, post};
use serde::{Deserialize, Serialize};
use skp_validator::Validate;
use skp_validator_actix::ValidatedJson;

#[derive(Debug, Deserialize, Serialize, Validate)]
struct User {
    #[validate(length(min = 3))]
    name: String,
    
    #[validate(range(min = 18))]
    age: u32,
}

#[post("/users")]
async fn create_user(user: ValidatedJson<User>) -> HttpResponse {
    HttpResponse::Ok().json(user.into_inner())
}

#[actix_rt::test]
async fn test_valid_request() {
    let mut app = test::init_service(App::new().service(create_user)).await;
    
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&User { name: "John".to_string(), age: 25 })
        .to_request();
        
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_invalid_request() {
    let mut app = test::init_service(App::new().service(create_user)).await;
    
    // Invalid name and age
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&User { name: "Jo".to_string(), age: 10 })
        .to_request();
        
    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
    
    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    println!("Error body: {}", body_str);
    
    // Check if body contains specific errors
    // If serde is enabled, it should be JSON. If not, maybe string representation.
    // skp-validator-core has Display impl too.
    assert!(body_str.contains("length") || body_str.contains("invalid length"));
    assert!(body_str.contains("range") || body_str.contains("out of range"));
}
