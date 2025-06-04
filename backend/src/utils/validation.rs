use regex::Regex;
use std::sync::OnceLock;

// Constants for input length limits
pub const MAX_NAME_LENGTH: usize = 255;
pub const MAX_EMAIL_LENGTH: usize = 255;
pub const MAX_TITLE_LENGTH: usize = 500;
pub const MAX_DESCRIPTION_LENGTH: usize = 2000;
pub const MAX_CONTENT_LENGTH: usize = 100_000; // 100KB for content
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_PASSWORD_LENGTH: usize = 128;
pub const MAX_ROLE_LENGTH: usize = 50;
pub const MAX_PRONOUNS_LENGTH: usize = 50;
pub const MAX_URL_LENGTH: usize = 500;

// Global regex instance using OnceLock for thread safety
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

/// Validation error types
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

/// Validation result type
pub type ValidationResult = Result<(), ValidationError>;

/// Initialize the email regex pattern
fn get_email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        // RFC 5322 compliant email regex (simplified but robust)
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Invalid email regex pattern")
    })
}

/// Validate email format and length
pub fn validate_email(email: &str, field_name: &str) -> ValidationResult {
    // Check if empty
    if email.trim().is_empty() {
        return Err(ValidationError::new(field_name, "Email is required"));
    }

    // Check length
    if email.len() > MAX_EMAIL_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("Email must be less than {} characters", MAX_EMAIL_LENGTH),
        ));
    }

    // Check format
    if !get_email_regex().is_match(email) {
        return Err(ValidationError::new(
            field_name,
            "Invalid email format",
        ));
    }

    Ok(())
}

/// Validate name fields (user name, project name, etc.)
pub fn validate_name(name: &str, field_name: &str) -> ValidationResult {
    let trimmed = name.trim();
    
    // Check if empty
    if trimmed.is_empty() {
        return Err(ValidationError::new(field_name, &format!("{} is required", field_name)));
    }

    // Check length
    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("{} must be less than {} characters", field_name, MAX_NAME_LENGTH),
        ));
    }

    // Check for potentially malicious characters (control characters)
    if trimmed.chars().any(|c| {
        c == '\0' || 
        (c >= '\x01' && c <= '\x08') || 
        (c >= '\x0B' && c <= '\x0C') || 
        (c >= '\x0E' && c <= '\x1F')
    }) {
        return Err(ValidationError::new(
            field_name,
            "Name contains invalid characters",
        ));
    }

    Ok(())
}

/// Validate password strength and length
pub fn validate_password(password: &str, field_name: &str) -> ValidationResult {
    // Check minimum length
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("Password must be at least {} characters long", MIN_PASSWORD_LENGTH),
        ));
    }

    // Check maximum length
    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("Password must be less than {} characters", MAX_PASSWORD_LENGTH),
        ));
    }

    // Check for null bytes and other control characters
    if password.contains('\0') {
        return Err(ValidationError::new(
            field_name,
            "Password contains invalid characters",
        ));
    }

    Ok(())
}

/// Validate title fields (ticket title, etc.)
pub fn validate_title(title: &str, field_name: &str) -> ValidationResult {
    let trimmed = title.trim();
    
    // Check if empty
    if trimmed.is_empty() {
        return Err(ValidationError::new(field_name, &format!("{} is required", field_name)));
    }

    // Check length
    if trimmed.len() > MAX_TITLE_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("{} must be less than {} characters", field_name, MAX_TITLE_LENGTH),
        ));
    }

    Ok(())
}

/// Validate description fields
pub fn validate_description(description: &str, field_name: &str) -> ValidationResult {
    // Description can be empty, but if provided, check length
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("{} must be less than {} characters", field_name, MAX_DESCRIPTION_LENGTH),
        ));
    }

    Ok(())
}

/// Validate content fields (article content, comments, etc.)
pub fn validate_content(content: &str, field_name: &str) -> ValidationResult {
    // Check if empty
    if content.trim().is_empty() {
        return Err(ValidationError::new(field_name, &format!("{} is required", field_name)));
    }

    // Check length
    if content.len() > MAX_CONTENT_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("{} must be less than {} characters", field_name, MAX_CONTENT_LENGTH),
        ));
    }

    Ok(())
}

/// Validate role fields
pub fn validate_role(role: &str, field_name: &str) -> ValidationResult {
    let trimmed = role.trim();
    
    // Check if empty
    if trimmed.is_empty() {
        return Err(ValidationError::new(field_name, "Role is required"));
    }

    // Check length
    if trimmed.len() > MAX_ROLE_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("Role must be less than {} characters", MAX_ROLE_LENGTH),
        ));
    }

    // Validate against allowed roles
    match trimmed.to_lowercase().as_str() {
        "admin" | "technician" | "user" => Ok(()),
        _ => Err(ValidationError::new(
            field_name,
            "Invalid role. Must be 'admin', 'technician', or 'user'",
        )),
    }
}

/// Validate pronouns field
pub fn validate_pronouns(pronouns: &str, field_name: &str) -> ValidationResult {
    // Pronouns are optional, but if provided, check length
    if pronouns.len() > MAX_PRONOUNS_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("Pronouns must be less than {} characters", MAX_PRONOUNS_LENGTH),
        ));
    }

    Ok(())
}

/// Validate URL fields (avatar_url, banner_url, etc.)
pub fn validate_url(url: &str, field_name: &str) -> ValidationResult {
    // URL is optional, but if provided, validate
    if url.is_empty() {
        return Ok(());
    }

    // Check length
    if url.len() > MAX_URL_LENGTH {
        return Err(ValidationError::new(
            field_name,
            &format!("URL must be less than {} characters", MAX_URL_LENGTH),
        ));
    }

    // Basic URL format validation
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("/") {
        return Err(ValidationError::new(
            field_name,
            "URL must start with http://, https://, or / (relative path)",
        ));
    }

    Ok(())
}

/// Validate UUID format
pub fn validate_uuid(uuid: &str, field_name: &str) -> ValidationResult {
    // Check if empty
    if uuid.trim().is_empty() {
        return Err(ValidationError::new(field_name, &format!("{} is required", field_name)));
    }

    // Try to parse as UUID
    if uuid::Uuid::parse_str(uuid).is_err() {
        return Err(ValidationError::new(
            field_name,
            "Invalid UUID format",
        ));
    }

    Ok(())
}

/// Validate positive integer
pub fn validate_positive_integer(value: i32, field_name: &str) -> ValidationResult {
    if value <= 0 {
        return Err(ValidationError::new(
            field_name,
            &format!("{} must be a positive integer", field_name),
        ));
    }

    Ok(())
}

/// Validate pagination parameters
pub fn validate_pagination(page: i64, page_size: i64) -> Result<(i64, i64), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let validated_page = if page < 1 {
        errors.push(ValidationError::new("page", "Page must be at least 1"));
        1
    } else {
        page
    };

    let validated_page_size = if page_size < 1 {
        errors.push(ValidationError::new("pageSize", "Page size must be at least 1"));
        25
    } else if page_size > 100 {
        errors.push(ValidationError::new("pageSize", "Page size cannot exceed 100"));
        100
    } else {
        page_size
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok((validated_page, validated_page_size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(validate_email("test@example.com", "email").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk", "email").is_ok());
        
        // Invalid emails
        assert!(validate_email("", "email").is_err());
        assert!(validate_email("invalid.email", "email").is_err());
        assert!(validate_email("@domain.com", "email").is_err());
        assert!(validate_email("user@", "email").is_err());
        
        // Too long email
        let long_email = format!("{}@example.com", "a".repeat(300));
        assert!(validate_email(&long_email, "email").is_err());
    }

    #[test]
    fn test_password_validation() {
        // Valid passwords
        assert!(validate_password("password123", "password").is_ok());
        assert!(validate_password("VerySecure!Password", "password").is_ok());
        
        // Invalid passwords
        assert!(validate_password("short", "password").is_err());
        assert!(validate_password("", "password").is_err());
        
        // Too long password
        let long_password = "a".repeat(200);
        assert!(validate_password(&long_password, "password").is_err());
    }

    #[test]
    fn test_role_validation() {
        // Valid roles
        assert!(validate_role("admin", "role").is_ok());
        assert!(validate_role("technician", "role").is_ok());
        assert!(validate_role("user", "role").is_ok());
        assert!(validate_role("ADMIN", "role").is_ok()); // Case insensitive
        
        // Invalid roles
        assert!(validate_role("", "role").is_err());
        assert!(validate_role("superuser", "role").is_err());
        assert!(validate_role("invalid", "role").is_err());
    }
} 