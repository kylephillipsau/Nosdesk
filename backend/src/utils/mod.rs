pub mod auth;
pub mod user;
pub mod validation;
pub mod image;
pub mod jwt;
pub mod sse;

use uuid::Uuid;
use crate::models::{UserRole, UserInfo};

/// Custom error types for better error handling
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(String),
    #[error("Invalid role: {0}. Must be 'admin', 'technician', or 'user'")]
    InvalidRole(String),
    #[error("Validation failed: {0}")]
    General(String),
}

/// Result type alias for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Parse UUID from string with proper error handling
pub fn parse_uuid(uuid_str: &str) -> ValidationResult<Uuid> {
    Uuid::parse_str(uuid_str)
        .map_err(|_| ValidationError::InvalidUuid(uuid_str.to_string()))
}

/// Parse optional UUID from string  
pub fn parse_optional_uuid(uuid_str: Option<&str>) -> ValidationResult<Option<Uuid>> {
    match uuid_str {
        Some(s) if !s.is_empty() => Ok(Some(parse_uuid(s)?)),
        _ => Ok(None),
    }
}

/// Convert UUID to string safely
pub fn uuid_to_string(uuid: &Uuid) -> String {
    uuid.to_string()
}

/// Convert optional UUID to optional string
pub fn optional_uuid_to_string(uuid: &Option<Uuid>) -> Option<String> {
    uuid.as_ref().map(|u| u.to_string())
}

/// Convert UserRole enum to string for JWT and API responses
pub fn role_to_string(role: &UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Technician => "technician".to_string(),
        UserRole::User => "user".to_string(),
    }
}

/// Parse string to UserRole enum
pub fn parse_role(role_str: &str) -> ValidationResult<UserRole> {
    match role_str.trim().to_lowercase().as_str() {
        "admin" => Ok(UserRole::Admin),
        "technician" => Ok(UserRole::Technician),
        "user" => Ok(UserRole::User),
        _ => Err(ValidationError::InvalidRole(role_str.to_string())),
    }
}

/// Normalize and trim string input
pub fn normalize_string(input: &str) -> String {
    input.trim().to_string()
}

/// Normalize email (trim + lowercase)
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Convert bytes to string for content fields
pub fn bytes_to_string(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| "Invalid UTF-8 content".to_string())
}

/// Convert string to bytes for content fields
pub fn string_to_bytes(content: &str) -> Vec<u8> {
    content.as_bytes().to_vec()
}

/// Create UserInfo from UUID and name (helper for responses)
pub fn create_user_info(uuid: Uuid, name: String) -> UserInfo {
    UserInfo { uuid, name }
}

/// Safe string unwrapping for Option<String> fields
pub fn unwrap_optional_string(opt_str: &Option<String>) -> String {
    opt_str.as_ref().unwrap_or(&String::new()).clone()
}

/// Wrap string in Some() for Option<String> fields
pub fn wrap_string(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

pub use auth::*;
pub use user::*;
pub use validation::*;
pub use image::*;
pub use jwt::*;
pub use sse::*; 