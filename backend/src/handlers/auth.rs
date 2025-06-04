use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::verify;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use actix_web::middleware::{Logger, DefaultHeaders};
use actix_web::http::header::HeaderValue;

use crate::db::DbConnection;
use crate::models::{
    Claims, LoginRequest, LoginResponse, NewUser, PasswordChangeRequest,
    UserRegistration, UserResponse, UserRole
};
use crate::repository;
use crate::utils::{self, ValidationResult, ValidationError, parse_uuid, uuid_to_string};
use crate::utils::auth::{hash_password, create_local_auth_identity, update_auth_identity_password};
use diesel::prelude::*;
use crate::schema::user_auth_identities;

// JWT secret key
lazy_static::lazy_static! {
    pub static ref JWT_SECRET: String = 
        std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set");
}

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

/// Helper to create a JWT token for a user
fn create_jwt_token(user: &crate::models::User) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        sub: uuid_to_string(&user.uuid),
        name: user.name.clone(),
        email: user.email.clone(),
        role: utils::role_to_string(&user.role),
        exp: now + 24 * 60 * 60, // 24 hours from now
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
}

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

    // Get local auth identity for this user
    let auth_identities = match repository::user_auth_identities::get_user_identities(user.id, &mut conn) {
        Ok(identities) => identities,
        Err(e) => {
            eprintln!("Error fetching auth identities: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error processing credentials"
            }));
        }
    };

    // Find local identity with password hash
    let local_identity = auth_identities.iter().find(|identity| identity.provider_id == 1);
    
    // Check if local identity exists
    let local_identity = match local_identity {
        Some(identity) => identity,
        None => {
            eprintln!("No local auth identity found for user: {}", user.id);
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };
    
    // Check if password hash exists
    let password_hash = match &local_identity.password_hash {
        Some(hash) => hash,
        None => {
            eprintln!("No password hash found for local auth identity: {}", local_identity.id);
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid email or password"
            }));
        }
    };

    // Verify password (bcrypt stores as string, not bytes)
    let password_matches = match verify(&login_data.password, password_hash) {
        Ok(matches) => matches,
        Err(_) => false,
    };

    if !password_matches {
        return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid email or password"
        }));
    }

    // Generate JWT token
    let token = match create_jwt_token(&user) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error generating token"
        })),
    };

    // Return token and user info
    HttpResponse::Ok().json(LoginResponse {
        token,
        user: user.into(),
    })
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

    // Create new user with proper password hash as bytes
    let new_user = NewUser {
        uuid: user_uuid,
        name: utils::normalize_string(&user_data.name),
        email: utils::normalize_email(&user_data.email),
        role: user_role,
        password_hash: password_hash.as_bytes().to_vec(), // Convert string to bytes for model
        pronouns: user_data.pronouns.as_ref().map(|p| utils::normalize_string(p)),
        avatar_url: user_data.avatar_url.as_ref().map(|u| utils::normalize_string(u)),
        banner_url: user_data.banner_url.as_ref().map(|u| utils::normalize_string(u)),
        avatar_thumb: user_data.avatar_thumb.as_ref().map(|u| utils::normalize_string(u)),
        microsoft_uuid: None,
    };

    // Save user to database
    match repository::create_user(new_user, &mut conn) {
        Ok(created_user) => {
            // Create local auth identity for the user
            let mut new_identity = create_local_auth_identity(
                created_user.id,
                password_hash,
                user_uuid,
            );
            new_identity.email = Some(created_user.email.clone());
            
            match repository::user_auth_identities::create_identity(new_identity, &mut conn) {
                Ok(_) => {
                    println!("✅ New user registered successfully: {}", created_user.email);
                    HttpResponse::Created().json(UserResponse::from(created_user))
                },
                Err(e) => {
                    eprintln!("Error creating user identity: {:?}", e);
                    // If identity creation fails, attempt to delete the user
                    let _ = repository::delete_user(created_user.id, &mut conn);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error creating user identity"
                    }))
                },
            }
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
    let token = auth.token();
    
    // Decode and validate token
    let claims = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data.claims,
        Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid token")),
    };
    
    // Parse the UUID from the JWT claims
    let user_uuid = match parse_uuid(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid user UUID in token")),
    };
    
    // Get user from database to ensure they still exist
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Database error")),
    };
    
    let user = match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => user,
        Err(_) => return Err(actix_web::error::ErrorUnauthorized("User not found")),
    };
    
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
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(e) => return e.into(),
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
            // Get local auth identity for this user
            let auth_identities = match repository::user_auth_identities::get_user_identities(user.id, &mut conn) {
                Ok(identities) => identities,
                Err(e) => {
                    eprintln!("Error fetching auth identities: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error processing credentials"
                    }));
                }
            };

            // Find local identity with password hash
            let local_identity = auth_identities.iter().find(|identity| identity.provider_id == 1);
            
            // Check if local identity exists
            let local_identity = match local_identity {
                Some(identity) => identity,
                None => {
                    eprintln!("No local auth identity found for user: {}", user.id);
                    return HttpResponse::BadRequest().json(json!({
                        "status": "error",
                        "message": "No local password found for this user"
                    }));
                }
            };
            
            // Check if password hash exists
            let password_hash = match &local_identity.password_hash {
                Some(hash) => hash,
                None => {
                    eprintln!("No password hash found for local auth identity: {}", local_identity.id);
                    return HttpResponse::BadRequest().json(json!({
                        "status": "error",
                        "message": "No password is set for this account"
                    }));
                }
            };

            // Verify current password
            let password_matches = match verify(&password_data.current_password, password_hash) {
                Ok(matches) => matches,
                Err(e) => {
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
            if verify(&password_data.new_password, password_hash).unwrap_or(false) {
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

            // Update the auth identity directly
            // Delete and re-insert the identity (simplified approach)
            match diesel::delete(
                user_auth_identities::table.filter(user_auth_identities::id.eq(local_identity.id))
            ).execute(&mut conn) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error deleting auth identity: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error updating password"
                    }));
                }
            }
            
            // Create new identity with updated password
            let new_auth_identity = update_auth_identity_password(local_identity, new_password_hash);
            
            match repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
                Ok(_) => {
                    println!("✅ Password changed successfully for user: {}", user.email);
                    HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": "Password changed successfully"
                    }))
                },
                Err(e) => {
                    eprintln!("Error creating updated auth identity: {:?}", e);
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

// Helper function to validate token internally
pub async fn validate_token_internal(auth: &BearerAuth, conn: &mut DbConnection) -> Result<Claims, actix_web::Error> {
    let token = auth.token();
    
    // Create validation with stricter requirements
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true; // Ensure token hasn't expired
    validation.validate_nbf = true; // Ensure token is not used before valid time
    validation.leeway = 30; // Allow 30 seconds of clock skew
    
    // Decode the token
    let token_data = match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error decoding token: {:?}", e);
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    return Err(actix_web::error::ErrorUnauthorized("Token has expired"));
                },
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token format"));
                },
                _ => {
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
                }
            }
        }
    };
    
    let claims = token_data.claims;
    
    // Parse the UUID from the JWT claims
    let user_uuid = match parse_uuid(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid user UUID in token")),
    };
    
    // Additional security: Check if user still exists and is active
    let user = match repository::get_user_by_uuid(&user_uuid, conn) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Error finding user by UUID during token validation: {:?}", e);
            return Err(actix_web::error::ErrorUnauthorized("User not found or inactive"));
        }
    };
    
    // Optional: Check if user role has changed
    let user_role_str = utils::role_to_string(&user.role);
    if claims.role != user_role_str {
        eprintln!("User role mismatch in token for user {}: token has '{}', db has '{}'", 
                 claims.sub, claims.role, user_role_str);
        return Err(actix_web::error::ErrorUnauthorized("Token role mismatch - please log in again"));
    }
    
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

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can reset passwords"
        }));
    }

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

    // Get local auth identity for this user
    let auth_identities = match repository::user_auth_identities::get_user_identities(user.id, &mut conn) {
        Ok(identities) => identities,
        Err(e) => {
            eprintln!("Error fetching auth identities: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error processing user identities"
            }));
        }
    };

    // Find local identity
    let local_identity = auth_identities.iter().find(|identity| identity.provider_id == 1);
    
    match local_identity {
        Some(identity) => {
            // Update existing local identity
            // Delete the old identity 
            match diesel::delete(
                user_auth_identities::table.filter(user_auth_identities::id.eq(identity.id))
            ).execute(&mut conn) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error deleting auth identity: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error updating password"
                    }));
                }
            }
            
            // Create new identity with updated password
            let new_auth_identity = update_auth_identity_password(identity, new_password_hash);
            
            match repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": "Password reset successfully"
                })),
                Err(e) => {
                    eprintln!("Error creating updated auth identity: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error updating password"
                    }))
                }
            }
        },
        None => {
            // Create new local identity if none exists
            let new_auth_identity = create_local_auth_identity(user.id, new_password_hash, user.uuid.clone());
            
            match repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": "Password reset successfully"
                })),
                Err(e) => {
                    eprintln!("Error creating auth identity: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error setting password"
                    }))
                }
            }
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

    match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
        Err(_) => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
    }
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

    // Create the admin user with trimmed data
    let new_user = NewUser {
        uuid: user_uuid,
        name: utils::normalize_string(&admin_data.name),
        email: utils::normalize_email(&admin_data.email),
        role: UserRole::Admin,
        password_hash: password_hash.as_bytes().to_vec(), // Convert string to bytes for model
        pronouns: None,
        avatar_url: None,
        banner_url: None,
        avatar_thumb: None,
        microsoft_uuid: None,
    };

    match repository::create_user(new_user, &mut conn) {
        Ok(created_user) => {
            // Create local auth identity for the admin user
            let mut new_identity = create_local_auth_identity(
                created_user.id,
                password_hash,
                user_uuid,
            );
            new_identity.email = Some(created_user.email.clone());
            
            match repository::user_auth_identities::create_identity(new_identity, &mut conn) {
                Ok(_) => {
                    println!("✅ Initial admin user created successfully: {}", created_user.email);
                    
                    let response = crate::models::AdminSetupResponse {
                        success: true,
                        message: "Initial admin user created successfully".to_string(),
                        user: Some(UserResponse::from(created_user)),
                    };
                    HttpResponse::Created().json(response)
                },
                Err(e) => {
                    eprintln!("Error creating admin auth identity: {:?}", e);
                    // If identity creation fails, attempt to delete the user
                    let _ = repository::delete_user(created_user.id, &mut conn);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error creating admin authentication"
                    }))
                },
            }
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