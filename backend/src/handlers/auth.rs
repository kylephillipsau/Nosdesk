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
use crate::utils::mfa;

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
            ValidationError::ValidationFailed(msg) => HttpResponse::InternalServerError().json(json!({
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

    // Check if user has MFA enabled - if so, require MFA verification
    if mfa::user_has_mfa_enabled(&user) {
        let response = jwt_helpers::create_mfa_required_response(user.uuid);
        return HttpResponse::Ok().json(response);
    }

    // Check MFA policy enforcement (for users without MFA enabled)
    if let Err(_policy_error) = mfa::validate_mfa_policy(&user).await {
        // Instead of blocking, offer MFA setup for users who need it
        let response = jwt_helpers::create_mfa_setup_required_response(user.uuid);
        return HttpResponse::Ok().json(response);
    }

    // Create standard login response (no MFA required)
    match jwt_helpers::create_login_response(user) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error_response) => error_response,
    }
}

/// MFA Login - Verify MFA token and complete login
pub async fn mfa_login(
    db_pool: web::Data<crate::db::Pool>,
    login_data: web::Json<crate::models::MfaLoginRequest>,
    request: actix_web::HttpRequest,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Find user by email (same as regular login)
    let user = match repository::get_user_by_email(&login_data.email, &mut conn) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

    // Verify password first
    let password_hash = match String::from_utf8(user.password_hash.clone()) {
        Ok(hash) => {
            if hash.is_empty() {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid email or password"
                }));
            }
            hash
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

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

    // Check that user actually has MFA enabled
    if !mfa::user_has_mfa_enabled(&user) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "MFA is not enabled for this account"
        }));
    }

    // Check rate limiting for MFA attempts
    if !mfa::check_mfa_rate_limit(&user.uuid).await {
        return HttpResponse::TooManyRequests().json(json!({
            "status": "error",
            "message": "Too many MFA attempts. Please try again later."
        }));
    }

    // Verify MFA token (TOTP or backup code)
    let mfa_result = match mfa::verify_mfa_token(&user.uuid, &login_data.mfa_token, &mut conn).await {
        Ok(result) => result,
        Err(e) => {
            // Log failed MFA attempt for security monitoring
            mfa::log_mfa_attempt(&user.uuid, false, "login", &request).await;
            
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("MFA verification failed: {}", e)
            }));
        }
    };

    if !mfa_result.is_valid {
        // Log failed MFA attempt
        mfa::log_mfa_attempt(&user.uuid, false, "login", &request).await;
        
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid MFA token"
        }));
    }

    // Log successful MFA attempt
    mfa::log_mfa_attempt(&user.uuid, true, "login", &request).await;

    // Create successful MFA login response
    match jwt_helpers::create_mfa_login_response(
        user,
        mfa_result.backup_code_used.is_some(),
        mfa_result.requires_backup_code_regeneration,
    ) {
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

    // Check if MFA_ENCRYPTION_KEY is set - use the util function
    if mfa::encrypt_mfa_secret("test").is_err() {
        tracing::error!("MFA setup failed: encryption key not configured");
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "MFA not properly configured on server"
        }));
    }

    // Generate new TOTP secret and backup codes (async hashing for security + performance)
    let secret = mfa::generate_totp_secret();
    let (backup_codes_plaintext, backup_codes_hashed) = mfa::generate_backup_codes_async().await;
    
    // Generate QR code
    let qr_code = match mfa::generate_qr_code(secret.as_str(), &user.email, "Nosdesk") {
        Ok(qr) => qr,
        Err(e) => {
            tracing::error!("Failed to generate QR code: {}", e);
            return HttpResponse::InternalServerError().json(json!({
            "status": "error",
                "message": "Failed to generate QR code"
            }));
        }
    };

    tracing::info!("MFA setup initiated for user: {}", user.uuid);

    let response = crate::models::MfaSetupResponse {
        secret: secret.as_str().to_string(),
        qr_code,
        backup_codes: backup_codes_plaintext,
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

    // Verify the TOTP token (timing-safe verification)
    if !mfa::verify_totp_token(&request.secret, &request.token) {
        tracing::warn!("MFA setup verification failed for user: {}", claims.sub);
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid verification code"
        }));
    }

    tracing::info!("MFA setup verification successful for user: {}", claims.sub);

    // Return success - backup codes were already generated in setup phase
    let response = crate::models::MfaVerifySetupResponse {
        success: true,
        backup_codes: vec![], // Empty - codes already provided in setup
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

    tracing::info!("Enabling MFA for user: {}", user_uuid);

    // Validate inputs securely
    let mfa_secret = match &request.secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "MFA secret is required"
        })),
    };

    let backup_codes = match &request.backup_codes {
        Some(codes) if !codes.is_empty() => codes,
        _ => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Backup codes are required"
        })),
    };

    // Final TOTP verification before enabling
    if !mfa::verify_totp_token(mfa_secret, &request.token) {
        tracing::warn!("MFA enable verification failed for user: {}", user_uuid);
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid verification code"
        }));
    }

    // Encrypt the MFA secret before storage
    let encrypted_secret = match mfa::encrypt_mfa_secret(mfa_secret) {
        Ok(encrypted) => encrypted,
        Err(e) => {
            tracing::error!("Failed to encrypt MFA secret: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to secure MFA data"
            }));
        }
    };

    // Hash backup codes with bcrypt (moved from setup to enable for performance)
    let mut hashed_backup_codes = Vec::new();
    for code in backup_codes {
        let hashed_code = match bcrypt::hash(code, bcrypt::DEFAULT_COST) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::error!("Failed to hash backup code: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to secure backup codes"
                }));
            }
        };
        hashed_backup_codes.push(hashed_code);
    }

    // Use the pre-hashed backup codes from setup phase
    // Note: backup_codes from frontend are plaintext from setup, but we hash them here
    // This maintains security while keeping the API simple
    
    let backup_codes_json = match serde_json::to_value(&hashed_backup_codes) {
        Ok(json) => json,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to serialize backup codes"
        })),
    };

    let mfa_update = crate::models::UserMfaUpdate {
        mfa_enabled: Some(true),
        mfa_secret: Some(encrypted_secret),
        mfa_backup_codes: Some(backup_codes_json),
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    match repository::update_user_mfa(&user_uuid, mfa_update, &mut conn) {
        Ok(_) => {
            tracing::info!("MFA enabled successfully for user: {}", user_uuid);
            HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "MFA enabled successfully"
            }))
        },
        Err(e) => {
            tracing::error!("Failed to enable MFA in database: {:?}", e);
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
    let (backup_codes_plaintext, backup_codes_hashed) = mfa::generate_backup_codes_async().await;
    let backup_codes_json = serde_json::to_value(&backup_codes_hashed).unwrap();

    let mfa_update = crate::models::UserMfaUpdate {
        mfa_enabled: None, // Don't change MFA enabled status
        mfa_secret: None,  // Don't change secret
        mfa_backup_codes: Some(backup_codes_json),
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    match repository::update_user_mfa(&user_uuid, mfa_update, &mut conn) {
        Ok(_) => {
            let response = crate::models::MfaRegenerateBackupCodesResponse {
                backup_codes: backup_codes_plaintext,
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

    // Check if user has backup codes
    let has_backup_codes = user.mfa_backup_codes
        .as_ref()
        .and_then(|codes| codes.as_array())
        .map(|array| !array.is_empty())
        .unwrap_or(false);

    let response = crate::models::MfaStatusResponse {
        enabled: user.mfa_enabled,
        has_backup_codes,
    };

    HttpResponse::Ok().json(response)
}



/// MFA Setup for Login (Unauthenticated) - For users who need MFA to login but haven't set it up yet
pub async fn mfa_setup_login(
    db_pool: web::Data<crate::db::Pool>,
    request: web::Json<crate::models::MfaSetupLoginRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Find user by email and verify password (same as login flow)
    let user = match repository::get_user_by_email(&request.email, &mut conn) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

    // Verify password
    let password_hash = match String::from_utf8(user.password_hash.clone()) {
        Ok(hash) => {
            if hash.is_empty() {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid email or password"
                }));
            }
            hash
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

    let password_matches = match verify(&request.password, &password_hash) {
        Ok(matches) => matches,
        Err(_) => false,
    };

    if !password_matches {
        return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid email or password"
        }));
    }

    // Verify that user actually needs MFA setup (security check)
    if mfa::user_has_mfa_enabled(&user) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "MFA is already enabled for this account"
        }));
    }

    // Verify that MFA is required for this user
    if let Ok(_) = mfa::validate_mfa_policy(&user).await {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "MFA is not required for this account"
        }));
    }

    // Check if MFA_ENCRYPTION_KEY is set
    if mfa::encrypt_mfa_secret("test").is_err() {
        tracing::error!("MFA setup failed: encryption key not configured");
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "MFA not properly configured on server"
        }));
    }

    // Generate new TOTP secret and backup codes
    let secret = mfa::generate_totp_secret();
    let (backup_codes_plaintext, _backup_codes_hashed) = mfa::generate_backup_codes_async().await;
    
    // Generate QR code
    let qr_code = match mfa::generate_qr_code(secret.as_str(), &user.email, "Nosdesk") {
        Ok(qr) => qr,
        Err(e) => {
            tracing::error!("Failed to generate QR code: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to generate QR code"
            }));
        }
    };

    tracing::info!("MFA setup initiated for user during login: {}", user.uuid);

    let response = crate::models::MfaSetupResponse {
        secret: secret.as_str().to_string(),
        qr_code,
        backup_codes: backup_codes_plaintext,
    };

    HttpResponse::Ok().json(response)
}

/// MFA Enable for Login (Unauthenticated) - Complete MFA setup and login
pub async fn mfa_enable_login(
    db_pool: web::Data<crate::db::Pool>,
    request: web::Json<crate::models::MfaEnableLoginRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Find user by email and verify password again (security)
    let user = match repository::get_user_by_email(&request.email, &mut conn) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

    // Verify password
    let password_hash = match String::from_utf8(user.password_hash.clone()) {
        Ok(hash) => {
            if hash.is_empty() {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid email or password"
                }));
            }
            hash
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

    let password_matches = match verify(&request.password, &password_hash) {
        Ok(matches) => matches,
        Err(_) => false,
    };

    if !password_matches {
        return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid email or password"
        }));
    }

    // Security checks - same as setup
    if mfa::user_has_mfa_enabled(&user) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "MFA is already enabled for this account"
        }));
    }

    if let Ok(_) = mfa::validate_mfa_policy(&user).await {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "MFA is not required for this account"
        }));
    }

    // Validate inputs securely
    let mfa_secret = match &request.secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "MFA secret is required"
        })),
    };

    let backup_codes = match &request.backup_codes {
        Some(codes) if !codes.is_empty() => codes,
        _ => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Backup codes are required"
        })),
    };

    // Final TOTP verification before enabling
    if !mfa::verify_totp_token(mfa_secret, &request.token) {
        tracing::warn!("MFA enable verification failed for user during login: {}", user.uuid);
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid verification code"
        }));
    }

    // Encrypt the MFA secret before storage
    let encrypted_secret = match mfa::encrypt_mfa_secret(mfa_secret) {
        Ok(encrypted) => encrypted,
        Err(e) => {
            tracing::error!("Failed to encrypt MFA secret: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to secure MFA data"
            }));
        }
    };

    // Hash backup codes with bcrypt
    let mut hashed_backup_codes = Vec::new();
    for code in backup_codes {
        let hashed_code = match bcrypt::hash(code, bcrypt::DEFAULT_COST) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::error!("Failed to hash backup code: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to secure backup codes"
                }));
            }
        };
        hashed_backup_codes.push(hashed_code);
    }

    let backup_codes_json = match serde_json::to_value(&hashed_backup_codes) {
        Ok(json) => json,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to serialize backup codes"
        })),
    };

    // Enable MFA in database
    let mfa_update = crate::models::UserMfaUpdate {
        mfa_enabled: Some(true),
        mfa_secret: Some(encrypted_secret),
        mfa_backup_codes: Some(backup_codes_json),
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    match repository::update_user_mfa(&user.uuid, mfa_update, &mut conn) {
        Ok(_) => {
            tracing::info!("MFA enabled successfully for user during login: {}", user.uuid);
            
            // Generate JWT token and complete login
            match jwt_helpers::create_login_response(user) {
                Ok(response) => HttpResponse::Ok().json(response),
                Err(error_response) => error_response,
            }
        },
        Err(e) => {
            tracing::error!("Failed to enable MFA in database: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to enable MFA"
            }))
        }
    }
}

/// Enhanced MFA reset with multiple verification steps (OWASP recommendations)
/// This function demonstrates a secure MFA reset procedure
async fn initiate_secure_mfa_reset(
    user_uuid: &uuid::Uuid, 
    password: &str,
    conn: &mut DbConnection
) -> Result<String, String> {
    // Step 1: Verify current password
    let user = repository::get_user_by_uuid(user_uuid, conn)
        .map_err(|_| "User not found")?;
    
    let password_hash_str = String::from_utf8(user.password_hash.clone())
        .map_err(|_| "Error reading password hash")?;
    
    if !bcrypt::verify(password, &password_hash_str).unwrap_or(false) {
        return Err("Invalid password".to_string());
    }
    
    // Step 2: Generate secure reset token
    use rand::Rng;
    let reset_token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    // Step 3: In production, you would:
    // - Store reset token in database with expiry (15 minutes)
    // - Send email with reset link containing token
    // - Require user to click email link AND re-enter password
    // - Log the reset attempt for security monitoring
    // - Consider requiring admin approval for high-privilege users
    
    tracing::warn!("MFA reset initiated for user: {} (secure procedures should be implemented)", user_uuid);
    
    Ok(reset_token)
}