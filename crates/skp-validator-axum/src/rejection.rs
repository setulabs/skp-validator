//! Rejection types for validation failures.

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use skp_validator_core::ValidationErrors;

/// Rejection type for JSON parsing failures.
#[derive(Debug)]
pub enum JsonRejection {
    /// Failed to parse JSON
    JsonDataError(String),
    /// Missing Content-Type header
    MissingContentType,
    /// Invalid Content-Type header
    InvalidContentType(String),
    /// Failed to buffer request body
    BodyError(String),
}

impl std::fmt::Display for JsonRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonDataError(e) => write!(f, "Failed to parse JSON: {}", e),
            Self::MissingContentType => write!(f, "Missing Content-Type header"),
            Self::InvalidContentType(ct) => write!(f, "Invalid Content-Type: {}, expected application/json", ct),
            Self::BodyError(e) => write!(f, "Failed to read request body: {}", e),
        }
    }
}

impl std::error::Error for JsonRejection {}

impl IntoResponse for JsonRejection {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            Self::JsonDataError(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            Self::MissingContentType => (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string()),
            Self::InvalidContentType(_) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string()),
            Self::BodyError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let json_body = serde_json::json!({
            "error": "json_parse_error",
            "message": body
        });

        (status, axum::Json(json_body)).into_response()
    }
}

/// Rejection type for validation failures.
#[derive(Debug)]
pub struct ValidationRejection {
    /// The validation errors
    pub errors: ValidationErrors,
}

impl ValidationRejection {
    /// Create a new validation rejection
    pub fn new(errors: ValidationErrors) -> Self {
        Self { errors }
    }

    /// Get the validation errors
    pub fn errors(&self) -> &ValidationErrors {
        &self.errors
    }
}

impl std::fmt::Display for ValidationRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation failed: {}", self.errors)
    }
}

impl std::error::Error for ValidationRejection {}

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        let status = StatusCode::UNPROCESSABLE_ENTITY;
        
        // Convert errors to a structured JSON response
        let error_body = serde_json::json!({
            "error": "validation_error",
            "message": "Validation failed",
            "details": self.errors.to_flat_map()
        });

        (status, axum::Json(error_body)).into_response()
    }
}

/// Combined rejection for ValidatedJson extractor.
#[derive(Debug)]
pub enum ValidatedJsonRejection {
    /// JSON parsing failed
    Json(JsonRejection),
    /// Validation failed
    Validation(ValidationRejection),
}

impl std::fmt::Display for ValidatedJsonRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "{}", e),
            Self::Validation(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ValidatedJsonRejection {}

impl IntoResponse for ValidatedJsonRejection {
    fn into_response(self) -> Response {
        match self {
            Self::Json(e) => e.into_response(),
            Self::Validation(e) => e.into_response(),
        }
    }
}

impl From<JsonRejection> for ValidatedJsonRejection {
    fn from(e: JsonRejection) -> Self {
        Self::Json(e)
    }
}

impl From<ValidationRejection> for ValidatedJsonRejection {
    fn from(e: ValidationRejection) -> Self {
        Self::Validation(e)
    }
}
