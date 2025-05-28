use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::db::DbConnection;
use crate::models::{
    Claims, LoginRequest, LoginResponse, NewUser, PasswordChangeRequest,
    UserRegistration, UserResponse, UserRole
};
use crate::repository;
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
    let local_identity = auth_identities.iter().find(|identity| identity.auth_provider_id == 1);
    
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

    // Convert bytea to string for verification
    let hash_str = match std::str::from_utf8(password_hash) {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("Error converting hash to string");
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error processing credentials"
            }));
        }
    };
    
    // Verify password
    let password_matches = match verify(&login_data.password, &hash_str) {
        Ok(matches) => {
            eprintln!("Password verification result: {}", matches);
            matches
        },
        Err(e) => {
            eprintln!("Error verifying password: {:?}", e);
            false
        }
    };

    if !password_matches {
        return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid email or password"
        }));
    }

    // Generate JWT token
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        sub: user.uuid.clone(),
        name: user.name.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp: now + 24 * 60 * 60, // 24 hours from now
        iat: now,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    ) {
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

    // Check if user with this email already exists
    if let Ok(_) = repository::get_user_by_email(&user_data.email, &mut conn) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "User with this email already exists"
        }));
    }

    // Hash the password
    let password_hash = match hash(&user_data.password, DEFAULT_COST) {
        Ok(hash) => hash.into_bytes(),
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error hashing password"
        })),
    };

    // Generate a UUID if not provided
    let uuid = Uuid::new_v4().to_string();

    // Create new user without password hash
    let new_user = NewUser {
        uuid: uuid.clone(),
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        role: user_data.role.clone(),
        pronouns: user_data.pronouns.clone(),
        avatar_url: user_data.avatar_url.clone(),
        banner_url: user_data.banner_url.clone(),
        avatar_thumb: user_data.avatar_thumb.clone(),
        microsoft_uuid: None,
    };

    // Save user to database
    match repository::create_user(new_user, &mut conn) {
        Ok(created_user) => {
            // Create local auth identity for the user
            use crate::models::NewUserAuthIdentity;
            let new_identity = NewUserAuthIdentity {
                user_id: created_user.id,
                auth_provider_id: 1, // Assuming 1 is the ID for local provider
                provider_user_id: uuid.clone(), // Using UUID as the provider user ID for local auth
                email: Some(created_user.email.clone()),
                identity_data: None,
                password_hash: Some(password_hash),
            };
            
            match repository::user_auth_identities::create_identity(new_identity, &mut conn) {
                Ok(_) => HttpResponse::Created().json(UserResponse::from(created_user)),
                Err(_) => {
                    // If identity creation fails, attempt to delete the user
                    let _ = repository::delete_user(created_user.id, &mut conn);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error creating user identity"
                    }))
                },
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error creating user"
        })),
    }
}

// Middleware for validating JWT tokens
pub async fn validate_token(auth: BearerAuth, db_pool: web::Data<crate::db::Pool>) -> Result<UserResponse, actix_web::Error> {
    let token = auth.token();
    
    // Decode and validate token
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid token")),
    };
    
    let claims = token_data.claims;
    
    // Get user from database to ensure they still exist
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Database error")),
    };
    
    let user = match repository::get_user_by_uuid(&claims.sub, &mut conn) {
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
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get user info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Get the user from the database
    let user = match repository::get_user_by_uuid(&claims.sub, &mut conn) {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
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
    let local_identity = auth_identities.iter().find(|identity| identity.auth_provider_id == 1);
    
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

    // Convert bytea to string for verification
    let hash_str = match std::str::from_utf8(password_hash) {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("Error converting hash to string");
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error processing credentials"
            }));
        }
    };

    // Verify current password
    let password_matches = match verify(&password_data.current_password, &hash_str) {
        Ok(matches) => matches,
        Err(e) => {
            eprintln!("Error verifying password: {:?}", e);
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

    // Hash the new password
    let new_password_hash = match hash(&password_data.new_password, DEFAULT_COST) {
        Ok(hash) => hash.into_bytes(),
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error hashing new password"
        })),
    };

    // Update the auth identity directly
    let mut updated_identity = local_identity.clone();
    updated_identity.password_hash = Some(new_password_hash.clone());
    
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
    use crate::models::NewUserAuthIdentity;
    let new_auth_identity = NewUserAuthIdentity {
        user_id: user.id,
        auth_provider_id: local_identity.auth_provider_id,
        provider_user_id: local_identity.provider_user_id.clone(),
        email: local_identity.email.clone(),
        identity_data: local_identity.identity_data.clone(),
        password_hash: Some(new_password_hash),
    };
    
    match repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Password changed successfully"
        })),
        Err(e) => {
            eprintln!("Error creating updated auth identity: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating password"
            }))
        }
    }
}

// Helper function to validate token internally
pub async fn validate_token_internal(auth: &BearerAuth, conn: &mut DbConnection) -> Result<Claims, actix_web::Error> {
    let token = auth.token();
    
    // Decode the token
    let token_data = match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error decoding token: {:?}", e);
            return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
        }
    };
    
    let claims = token_data.claims;
    
    // Check if user exists
    if let Err(e) = repository::get_user_by_uuid(&claims.sub, conn) {
        eprintln!("Error finding user by UUID: {:?}", e);
        return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
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
    let new_password_hash = match hash(&reset_data.new_password, DEFAULT_COST) {
        Ok(hash) => hash.into_bytes(),
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
    let local_identity = auth_identities.iter().find(|identity| identity.auth_provider_id == 1);
    
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
            use crate::models::NewUserAuthIdentity;
            let new_auth_identity = NewUserAuthIdentity {
                user_id: user.id,
                auth_provider_id: identity.auth_provider_id,
                provider_user_id: identity.provider_user_id.clone(),
                email: identity.email.clone(),
                identity_data: identity.identity_data.clone(),
                password_hash: Some(new_password_hash),
            };
            
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
            use crate::models::NewUserAuthIdentity;
            let new_auth_identity = NewUserAuthIdentity {
                user_id: user.id,
                auth_provider_id: 1, // Local provider
                provider_user_id: user.uuid.clone(), // Use UUID as provider user ID for local auth
                email: Some(user.email.clone()),
                identity_data: None,
                password_hash: Some(new_password_hash),
            };
            
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
    // Validate token and get user info
    match validate_token(auth, db_pool).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => {
            // Simply return a 401 Unauthorized for any token validation errors
            HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid or expired token"
            }))
        }
    }
} 