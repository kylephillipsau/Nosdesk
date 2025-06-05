use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::verify;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;


use crate::db::DbConnection;
use crate::models::{
    Claims, LoginRequest, PasswordChangeRequest,
    UserRegistration, UserResponse, UserRole
};
use crate::repository;
use crate::utils::{self, ValidationResult, ValidationError, parse_uuid};
use crate::utils::auth::hash_password;

// Import JWT utilities
use crate::utils::jwt::{JwtUtils, helpers as jwt_helpers};

// Admin password reset request
#[derive(Deserialize)]
pub struct AdminPasswordResetRequest {
    pub user_id: i32,
    pub new_password: String,
}



/// Convert ValidationError to HTTP response
impl From<ValidationError> for HttpResponse {
    fn from(error: ValidationError) -> Self {
        match error {
            ValidationError::InvalidUuid(_) => HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": error.to_string()
            })),
            ValidationError::InvalidRole(_) => HttpResponse::BadRequest().json(json!({
                "status": "error", 
                "message": error.to_string()
            })),
            ValidationError::General(msg) => HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": msg
            })),
        }
    }
}

// JWT token creation functions moved to jwt utils module



/// Helper function to parse UUID from string (like JWT claims)
fn parse_uuid_from_string(uuid_str: &str) -> Result<Uuid, String> {
    Uuid::parse_str(uuid_str).map_err(|_| "Invalid UUID format".to_string())
}

/// Helper function to convert UserRole enum to string for JWT
fn user_role_to_string(role: &UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Technician => "technician".to_string(),
        UserRole::User => "user".to_string(),
    }
}

/// Helper function to parse string to UserRole enum
fn parse_user_role(role_str: &str) -> Result<UserRole, String> {
    match role_str.to_lowercase().as_str() {
        "admin" => Ok(UserRole::Admin),
        "technician" => Ok(UserRole::Technician),
        "user" => Ok(UserRole::User),
        _ => Err("Invalid role".to_string()),
    }
}

// Authentication handlers
pub async fn login(
    db_pool: web::Data<crate::db::Pool>,
    login_data: web::Json<LoginRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Find user by email
    let user = match repository::get_user_by_email(&login_data.email, &mut conn) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Error finding user by email: {:?}", e);
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };



    // Check if user has a password hash for local authentication
    let password_hash = match String::from_utf8(user.password_hash.clone()) {
        Ok(hash) => {
            if hash.is_empty() {
                eprintln!("No password hash found for user: {}", user.id);
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid email or password"
                }));
            }
            hash
        }
        Err(_) => {
            eprintln!("Invalid password hash format for user: {}", user.id);
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

    // Verify password
    let password_matches = match verify(&login_data.password, &password_hash) {
        Ok(matches) => matches,
        Err(_) => false,
    };

    if !password_matches {
        return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid email or password"
        }));
    }

    // Create login response with JWT token using JWT utilities
    match jwt_helpers::create_login_response(user) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error_response) => error_response,
    }
}

pub async fn register(
    db_pool: web::Data<crate::db::Pool>,
    user_data: web::Json<UserRegistration>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Comprehensive input validation using our validation utilities
    let mut validation_errors = Vec::new();

    // Validate name
    let trimmed_name = user_data.name.trim();
    if trimmed_name.is_empty() {
        validation_errors.push("name: Name is required".to_string());
    } else if trimmed_name.len() > 255 {
        validation_errors.push("name: Name must be less than 255 characters".to_string());
    }

    // Validate email
    let trimmed_email = user_data.email.trim();
    if trimmed_email.is_empty() {
        validation_errors.push("email: Email is required".to_string());
    } else if trimmed_email.len() > 255 {
        validation_errors.push("email: Email must be less than 255 characters".to_string());
    } else if !trimmed_email.contains('@') || !trimmed_email.contains('.') {
        validation_errors.push("email: Invalid email format".to_string());
    }

    // Validate password
    if user_data.password.len() < 8 {
        validation_errors.push("password: Password must be at least 8 characters long".to_string());
    } else if user_data.password.len() > 128 {
        validation_errors.push("password: Password must be less than 128 characters".to_string());
    }

    // Validate role
    let trimmed_role = user_data.role.trim().to_lowercase();
    if !["admin", "technician", "user"].contains(&trimmed_role.as_str()) {
        validation_errors.push("role: Invalid role. Must be 'admin', 'technician', or 'user'".to_string());
    }

    // Validate optional fields
    if let Some(ref pronouns) = user_data.pronouns {
        if pronouns.len() > 50 {
            validation_errors.push("pronouns: Pronouns must be less than 50 characters".to_string());
        }
    }

    if let Some(ref avatar_url) = user_data.avatar_url {
        if avatar_url.len() > 500 {
            validation_errors.push("avatar_url: URL must be less than 500 characters".to_string());
        }
    }

    if let Some(ref banner_url) = user_data.banner_url {
        if banner_url.len() > 500 {
            validation_errors.push("banner_url: URL must be less than 500 characters".to_string());
        }
    }

    if let Some(ref avatar_thumb) = user_data.avatar_thumb {
        if avatar_thumb.len() > 500 {
            validation_errors.push("avatar_thumb: URL must be less than 500 characters".to_string());
        }
    }

    // If there are validation errors, return them
    if !validation_errors.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Validation failed",
            "errors": validation_errors
        }));
    }

    // Check if user with this email already exists
    if let Ok(_) = repository::get_user_by_email(&user_data.email, &mut conn) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "User with this email already exists"
        }));
    }

    // Hash the password
    let password_hash = match hash_password(&user_data.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error hashing password"
        })),
    };

    // Generate a UUID if not provided
    let user_uuid = Uuid::new_v4();

    // Parse role from string to enum
    let user_role = match utils::parse_role(&user_data.role) {
        Ok(role) => role,
        Err(e) => return e.into(),
    };

    // Create new user using builder pattern with normalized data
    let (normalized_name, normalized_email) = utils::normalization::normalize_user_data(&user_data.name, &user_data.email);
    let new_user = utils::NewUserBuilder::new(normalized_name, normalized_email, user_role)
        .with_uuid(user_uuid)
        .with_password_hash(password_hash.as_bytes().to_vec())
        .with_pronouns(utils::normalization::normalize_optional_string(user_data.pronouns.as_ref()))
        .with_avatar(
            utils::normalization::normalize_optional_string(user_data.avatar_url.as_ref()),
            utils::normalization::normalize_optional_string(user_data.avatar_thumb.as_ref())
        )
        .with_banner(utils::normalization::normalize_optional_string(user_data.banner_url.as_ref()))
        .build();

    // Save user to database
    match repository::create_user(new_user, &mut conn) {
        Ok(created_user) => {
            println!("✅ New user registered successfully: {}", created_user.email);
            HttpResponse::Created().json(UserResponse::from(created_user))
        },
        Err(e) => {
            eprintln!("Error creating user: {:?}", e);
            
            // Provide more specific error messages for common issues
            let error_message = if format!("{:?}", e).contains("duplicate") || format!("{:?}", e).contains("unique") {
                "Email address already exists in the system"
            } else {
                "Error creating user"
            };
            
            HttpResponse::InternalServerError().json(json!({
            "status": "error",
                "message": error_message
            }))
        },
    }
}

// Middleware for validating JWT tokens
pub async fn validate_token(auth: BearerAuth, db_pool: web::Data<crate::db::Pool>) -> Result<UserResponse, actix_web::Error> {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Database error")),
    };
    
    let (_claims, user) = JwtUtils::authenticate_request(&auth, &mut conn).await?;
    Ok(UserResponse::from(user))
}

pub async fn change_password(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    password_data: web::Json<PasswordChangeRequest>,
) -> impl Responder {
    // Validate new password first
    if password_data.new_password.len() < 8 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "New password validation failed: Password must be at least 8 characters long"
        }));
    } else if password_data.new_password.len() > 128 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "New password validation failed: Password must be less than 128 characters"
        }));
    }

    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get claims
    let (claims, _user) = match JwtUtils::authenticate_request(&auth, &mut conn).await {
        Ok((claims, user)) => (claims, user),
        Err(e) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Parse UUID from claims
    let user_uuid = match parse_uuid(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid user UUID in token"
        })),
    };

    match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => {
            // Check if user has a password hash for local authentication
            let current_password_hash = match String::from_utf8(user.password_hash.clone()) {
                Ok(hash) => {
                    if hash.is_empty() {
                        eprintln!("No password hash found for user: {}", user.id);
                        return HttpResponse::BadRequest().json(json!({
                            "status": "error",
                            "message": "No local password found for this user"
                        }));
                    }
                    hash
                }
                Err(_) => {
                    eprintln!("Invalid password hash format for user: {}", user.id);
                    return HttpResponse::BadRequest().json(json!({
                        "status": "error",
                        "message": "No password is set for this account"
                    }));
                }
            };

            // Verify current password
            let password_matches = match verify(&password_data.current_password, &current_password_hash) {
                Ok(matches) => matches,
                Err(_) => {
                    eprintln!("Error verifying current password during password change");
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error verifying password"
                    }));
                }
            };

            if !password_matches {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Current password is incorrect"
                }));
            }

            // Check if new password is the same as current password
            if verify(&password_data.new_password, &current_password_hash).unwrap_or(false) {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "New password must be different from current password"
                }));
            }

            // Hash the new password
            let new_password_hash = match hash_password(&password_data.new_password) {
                Ok(hash) => hash,
                Err(_) => return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Error hashing new password"
                })),
            };

            // Update the user's password hash directly
            use diesel::prelude::*;
            match diesel::update(crate::schema::users::table.find(user.id))
                .set(crate::schema::users::password_hash.eq(new_password_hash.as_bytes()))
                .execute(&mut conn) {
                Ok(_) => {
                    println!("✅ Password changed successfully for user: {}", user.email);
                    HttpResponse::Ok().json(json!({
                        "status": "success",
                        "message": "Password changed successfully"
                    }))
                },
                Err(e) => {
                    eprintln!("Error updating password: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error updating password"
                    }))
                }
            }
        },
        Err(_) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
    }
}

// Helper function to validate token internally - deprecated, use JwtUtils instead
pub async fn validate_token_internal(auth: &BearerAuth, conn: &mut DbConnection) -> Result<Claims, actix_web::Error> {
    let (claims, _user) = JwtUtils::authenticate_request(auth, conn).await?;
    Ok(claims)
}

pub async fn admin_reset_password(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    reset_data: web::Json<AdminPasswordResetRequest>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and check admin permissions
    let (claims, _user) = match jwt_helpers::require_admin(&auth, &mut conn).await {
        Ok((claims, user)) => (claims, user),
        Err(_) => return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can reset passwords"
        })),
    };

    // Get the target user
    let user = match repository::get_user_by_id(reset_data.user_id, &mut conn) {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
    };

    // Hash the new password
    let new_password_hash = match hash_password(&reset_data.new_password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error hashing new password"
        })),
    };

    // Update the user's password hash directly
    use diesel::prelude::*;
    match diesel::update(crate::schema::users::table.find(user.id))
        .set(crate::schema::users::password_hash.eq(new_password_hash.as_bytes()))
        .execute(&mut conn) {
        Ok(_) => {
            println!("✅ Password reset successfully for user: {}", user.email);
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Password reset successfully"
            }))
        },
        Err(e) => {
            eprintln!("Error resetting password: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating password"
            }))
        }
    }
}

// Handler to get current authenticated user
pub async fn get_current_user(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let (_claims, user) = match JwtUtils::authenticate_request(&auth, &mut conn).await {
        Ok((claims, user)) => (claims, user),
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    HttpResponse::Ok().json(UserResponse::from(user))
}

// Onboarding handlers for initial setup
pub async fn check_setup_status(
    db_pool: web::Data<crate::db::Pool>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    match repository::count_users(&mut conn) {
        Ok(user_count) => {
            let response = crate::models::OnboardingStatus {
                requires_setup: user_count == 0,
                user_count,
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            eprintln!("Error counting users: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to check setup status"
            }))
        }
    }
}

pub async fn setup_initial_admin(
    db_pool: web::Data<crate::db::Pool>,
    admin_data: web::Json<crate::models::AdminSetupRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Security check: Only allow setup if no users exist
    match repository::count_users(&mut conn) {
        Ok(user_count) => {
            if user_count > 0 {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Setup has already been completed. Users already exist in the system."
                }));
            }
        },
        Err(e) => {
            eprintln!("Error counting users: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to verify setup status"
            }));
        }
    }

    // Comprehensive input validation using our validation utilities
    let mut validation_errors = Vec::new();

    // Validate name
    let trimmed_name = admin_data.name.trim();
    if trimmed_name.is_empty() {
        validation_errors.push("name: Name is required".to_string());
    } else if trimmed_name.len() > 255 {
        validation_errors.push("name: Name must be less than 255 characters".to_string());
    }

    // Validate email
    let trimmed_email = admin_data.email.trim();
    if trimmed_email.is_empty() {
        validation_errors.push("email: Email is required".to_string());
    } else if trimmed_email.len() > 255 {
        validation_errors.push("email: Email must be less than 255 characters".to_string());
    } else if !trimmed_email.contains('@') || !trimmed_email.contains('.') {
        validation_errors.push("email: Invalid email format".to_string());
    }

    // Validate password
    if admin_data.password.len() < 8 {
        validation_errors.push("password: Password must be at least 8 characters long".to_string());
    } else if admin_data.password.len() > 128 {
        validation_errors.push("password: Password must be less than 128 characters".to_string());
    }

    // If there are validation errors, return them
    if !validation_errors.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Validation failed",
            "errors": validation_errors
        }));
    }

    // Hash the password
    let password_hash = match hash_password(&admin_data.password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Error hashing password: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error processing password"
            }));
        }
    };

    // Generate UUID for the admin user
    let user_uuid = Uuid::new_v4();

    // Create the admin user using convenience function
    let (normalized_name, normalized_email) = utils::normalization::normalize_user_data(&admin_data.name, &admin_data.email);
    let new_user = utils::NewUserBuilder::admin_user(
        normalized_name,
        normalized_email,
        password_hash.as_bytes().to_vec()
    ).build();

    match repository::create_user(new_user, &mut conn) {
        Ok(created_user) => {
            println!("✅ Initial admin user created successfully: {}", created_user.email);
            
            let response = crate::models::AdminSetupResponse {
                success: true,
                message: "Initial admin user created successfully".to_string(),
                user: Some(UserResponse::from(created_user)),
            };
            HttpResponse::Created().json(response)
        },
        Err(e) => {
            eprintln!("Error creating admin user: {:?}", e);
            
            // Provide more specific error messages for common issues
            let error_message = if format!("{:?}", e).contains("duplicate") || format!("{:?}", e).contains("unique") {
                "Email address already exists in the system"
            } else {
                "Error creating admin user"
            };
            
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": error_message
            }))
        }
    }
}

// === MFA (Multi-Factor Authentication) Handlers ===

use base32;
use base64::{Engine as _, engine::general_purpose};
use qrcode::{QrCode, render::svg};
use totp_rs::{Algorithm as TotpAlgorithm, TOTP, Secret};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Generate a random string for TOTP secret (32 characters, base32-encoded)
fn generate_totp_secret() -> String {
    let secret_bytes: Vec<u8> = (0..20).map(|_| thread_rng().gen()).collect(); // 20 bytes = 160 bits
    base32::encode(base32::Alphabet::RFC4648 { padding: true }, &secret_bytes)
}

/// Generate backup codes for MFA recovery
fn generate_backup_codes() -> Vec<String> {
    (0..8)
        .map(|_| {
            // Generate 8-character alphanumeric codes
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect::<String>()
                .to_uppercase()
        })
        .collect()
}

/// Generate QR code as SVG string
fn generate_qr_code(secret: &str, user_email: &str, service_name: &str) -> Result<String, String> {
    // Create TOTP URL for authenticator apps
    let totp_url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        service_name, user_email, secret, service_name
    );
    
    match QrCode::new(&totp_url) {
        Ok(code) => {
            let svg = code
                .render::<svg::Color>()
                .min_dimensions(200, 200)
                .build();
            
            // Convert SVG to base64 data URL for frontend
            let base64_svg = general_purpose::STANDARD.encode(svg);
            Ok(format!("data:image/svg+xml;base64,{}", base64_svg))
        }
        Err(e) => Err(format!("Failed to generate QR code: {}", e)),
    }
}

/// Verify TOTP token
fn verify_totp_token(secret: &str, token: &str) -> bool {
    match Secret::Encoded(secret.to_string()).to_bytes() {
        Ok(secret_bytes) => {
            let totp = TOTP::new(
                TotpAlgorithm::SHA1,
                6,        // 6-digit codes
                1,        // 1 step = 30 seconds
                30,       // 30-second window
                secret_bytes,
            ).unwrap();
            
            totp.check_current(token).unwrap_or(false)
        }
        Err(_) => false,
    }
}

/// MFA Setup - Generate secret and QR code
pub async fn mfa_setup(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let (_claims, user) = match JwtUtils::authenticate_request(&auth, &mut conn).await {
        Ok((claims, user)) => (claims, user),
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Generate new TOTP secret
    let secret = generate_totp_secret();
    let backup_codes = generate_backup_codes();
    
    // Generate QR code
    let qr_code = match generate_qr_code(&secret, &user.email, "Nosdesk") {
        Ok(qr) => qr,
        Err(e) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to generate QR code: {}", e)
        })),
    };

    let response = crate::models::MfaSetupResponse {
        secret,
        qr_code,
        backup_codes,
    };

    HttpResponse::Ok().json(response)
}

/// MFA Verify Setup - Verify the TOTP token during setup
pub async fn mfa_verify_setup(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    request: web::Json<crate::models::MfaVerifySetupRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Verify the TOTP token
    if !verify_totp_token(&request.secret, &request.token) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid verification code"
        }));
    }

    // Generate backup codes for successful setup
    let backup_codes = generate_backup_codes();

    let response = crate::models::MfaVerifySetupResponse {
        success: true,
        backup_codes,
    };

    HttpResponse::Ok().json(response)
}

/// MFA Enable - Enable MFA for the user
pub async fn mfa_enable(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    request: web::Json<crate::models::MfaEnableRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Parse UUID from claims
    let user_uuid = match parse_uuid(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid user UUID in token"
        })),
    };

    // Note: In a real implementation, you would store the secret and backup codes from the setup phase
    // For now, we'll just enable MFA flag
    let mfa_update = crate::models::UserMfaUpdate {
        mfa_enabled: Some(true),
        mfa_secret: None, // This should be set during the setup phase
        mfa_backup_codes: None, // This should be set during the setup phase
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    match repository::update_user_mfa(&user_uuid, mfa_update, &mut conn) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "MFA enabled successfully"
        })),
        Err(e) => {
            eprintln!("Error enabling MFA: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to enable MFA"
            }))
        }
    }
}

/// MFA Disable - Disable MFA for the user
pub async fn mfa_disable(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    request: web::Json<crate::models::MfaDisableRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Parse UUID from claims
    let user_uuid = match parse_uuid(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid user UUID in token"
        })),
    };

    // Get user to verify password
    let user = match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
    };

    // Verify password
    let password_hash_str = match String::from_utf8(user.password_hash.clone()) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error reading password hash"
        })),
    };

    if !verify(&request.password, &password_hash_str).unwrap_or(false) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid password"
        }));
    }

    // Disable MFA
    let mfa_update = crate::models::UserMfaUpdate {
        mfa_enabled: Some(false),
        mfa_secret: None, // Clear the secret
        mfa_backup_codes: Some(serde_json::Value::Null), // Clear backup codes
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    match repository::update_user_mfa(&user_uuid, mfa_update, &mut conn) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "MFA disabled successfully"
        })),
        Err(e) => {
            eprintln!("Error disabling MFA: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to disable MFA"
            }))
        }
    }
}

/// MFA Regenerate Backup Codes - Generate new backup codes
pub async fn mfa_regenerate_backup_codes(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    request: web::Json<crate::models::MfaRegenerateBackupCodesRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Parse UUID from claims
    let user_uuid = match parse_uuid(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid user UUID in token"
        })),
    };

    // Get user to verify password
    let user = match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
    };

    // Verify password
    let password_hash_str = match String::from_utf8(user.password_hash.clone()) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error reading password hash"
        })),
    };

    if !verify(&request.password, &password_hash_str).unwrap_or(false) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid password"
        }));
    }

    // Generate new backup codes
    let backup_codes = generate_backup_codes();
    let backup_codes_json = serde_json::to_value(&backup_codes).unwrap();

    let mfa_update = crate::models::UserMfaUpdate {
        mfa_enabled: None, // Don't change MFA enabled status
        mfa_secret: None,  // Don't change secret
        mfa_backup_codes: Some(backup_codes_json),
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    match repository::update_user_mfa(&user_uuid, mfa_update, &mut conn) {
        Ok(_) => {
            let response = crate::models::MfaRegenerateBackupCodesResponse {
                backup_codes,
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            eprintln!("Error regenerating backup codes: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to regenerate backup codes"
            }))
        }
    }
}

/// MFA Status - Get current MFA status for the user
pub async fn mfa_status(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Parse UUID from claims
    let user_uuid = match parse_uuid(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid user UUID in token"
        })),
    };

    // Get user MFA status
    let user = match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
    };

    // Check if user has backup codes (for now, we'll assume they do if MFA is enabled)
    let response = crate::models::MfaStatusResponse {
        enabled: false, // Will be updated when we have MFA fields in the User model
        has_backup_codes: false, // Will be updated when we have MFA fields in the User model
    };

    HttpResponse::Ok().json(response)
}