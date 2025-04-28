use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;
use serde_json::json;
use uuid::Uuid;
use futures::{StreamExt, TryStreamExt};
use actix_multipart::Multipart;
use std::fs;
use std::path::Path;

use crate::db::DbConnection;
use crate::models::{NewUser, User, UserResponse, UserUpdate, UserUpdateForm};
use crate::repository;
use crate::repository::user_auth_identities;
use crate::handlers::auth::validate_token_internal;

// User handlers
pub async fn get_users(
    pool: web::Data<crate::db::Pool>,
) -> impl Responder {
    println!("GET /api/users - Handler called");
    
    let mut conn = match pool.get() {
        Ok(conn) => {
            println!("Database connection obtained successfully");
            conn
        },
        Err(e) => {
            println!("Database connection error: {:?}", e);
            return HttpResponse::InternalServerError().json("Database connection error");
        },
    };

    println!("Calling repository::get_users");
    match repository::get_users(&mut conn) {
        Ok(users) => {
            println!("Successfully retrieved {} users", users.len());
            // Convert to UserResponse to hide sensitive information
            let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            println!("Converted to UserResponse, returning JSON");
            HttpResponse::Ok().json(user_responses)
        },
        Err(e) => {
            println!("Error fetching users: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to fetch users")
        },
    }
}

pub async fn get_user_by_id(
    id: web::Path<i32>,
    pool: web::Data<crate::db::Pool>,
) -> impl Responder {
    let user_id = id.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::get_user_by_id(user_id, &mut conn) {
        Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
        Err(_) => HttpResponse::NotFound().json("User not found"),
    }
}

pub async fn get_user_by_uuid(
    uuid: web::Path<String>,
    pool: web::Data<crate::db::Pool>,
) -> impl Responder {
    let user_uuid = uuid.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
        Err(_) => HttpResponse::NotFound().json("User not found"),
    }
}

pub async fn create_user(
    db_pool: web::Data<crate::db::Pool>,
    user_data: web::Json<NewUser>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    println!("Checking if user with email {} already exists", user_data.email);
    if let Ok(_) = repository::get_user_by_email(&user_data.email, &mut conn) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "User with this email already exists"
        }));
    }

    // Generate UUID if not provided
    let uuid = if user_data.uuid.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        user_data.uuid.clone()
    };

    // Create new user
    let new_user = NewUser {
        uuid: uuid.clone(),
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        role: user_data.role.clone(),
        pronouns: user_data.pronouns.clone(),
        avatar_url: user_data.avatar_url.clone(),
        banner_url: user_data.banner_url.clone(),
    };

    println!("Creating user with name {}", new_user.name);
    match repository::create_user(new_user, &mut conn) {
        Ok(user) => {
            println!("User created successfully");
            
            // Create default password hash for this user
            use bcrypt::hash;
            use crate::models::NewUserAuthIdentity;
            
            let password_hash = match hash("changeme", DEFAULT_COST) {
                Ok(hash) => hash.into_bytes(),
                Err(e) => {
                    println!("Error hashing password: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error setting default password"
                    }));
                }
            };
            
            // Create local auth identity with default password
            let new_identity = NewUserAuthIdentity {
                user_id: user.id,
                auth_provider_id: 1, // Local auth provider
                provider_user_id: uuid.clone(),
                email: Some(user.email.clone()),
                identity_data: None,
                password_hash: Some(password_hash),
            };
            
            match repository::user_auth_identities::create_identity(new_identity, &mut conn) {
                Ok(_) => HttpResponse::Created().json(UserResponse::from(user)),
                Err(e) => {
                    println!("Error creating auth identity: {:?}", e);
                    // If identity creation fails, still return the user
                    HttpResponse::Created().json(UserResponse::from(user))
                }
            }
        },
        Err(e) => {
            println!("Error creating user: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error creating user"
            }))
        }
    }
}

pub async fn update_user(
    db_pool: web::Data<crate::db::Pool>,
    path: web::Path<i32>,
    user_data: web::Json<UserUpdateForm>,
) -> impl Responder {
    let user_id = path.into_inner();
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    if let Err(_) = repository::get_user_by_id(user_id, &mut conn) {
        return HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        }));
    }

    // Check if email is being updated and if it's already in use
    if let Some(email) = &user_data.email {
        if let Ok(existing_user) = repository::get_user_by_email(email, &mut conn) {
            if existing_user.id != user_id {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Email already in use by another user"
                }));
            }
        }
    }

    // Create password hash if password is provided
    if let Some(password) = &user_data.password {
        use bcrypt::hash;
        use crate::models::NewUserAuthIdentity;
        
        // Hash the new password
        let password_hash = match hash(password, DEFAULT_COST) {
            Ok(hash) => hash.into_bytes(),
            Err(_) => return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error hashing password"
            })),
        };
        
        // Find existing local auth identity
        let auth_identities = match repository::user_auth_identities::get_user_identities(user_id, &mut conn) {
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
                    crate::schema::user_auth_identities::table.filter(crate::schema::user_auth_identities::id.eq(identity.id))
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
                let new_auth_identity = NewUserAuthIdentity {
                    user_id,
                    auth_provider_id: identity.auth_provider_id,
                    provider_user_id: identity.provider_user_id.clone(),
                    email: identity.email.clone(),
                    identity_data: identity.identity_data.clone(),
                    password_hash: Some(password_hash),
                };
                
                if let Err(e) = repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
                    eprintln!("Error creating updated auth identity: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error updating password"
                    }));
                }
            },
            None => {
                // Create new local identity if none exists
                let new_auth_identity = NewUserAuthIdentity {
                    user_id,
                    auth_provider_id: 1, // Local provider
                    provider_user_id: Uuid::new_v4().to_string(), // Generate a new provider user ID
                    email: user_data.email.clone(),
                    identity_data: None,
                    password_hash: Some(password_hash),
                };
                
                if let Err(e) = repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
                    eprintln!("Error creating auth identity: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error setting password"
                    }));
                }
            }
        }
    }

    // Update user
    let user_update = UserUpdate {
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        role: user_data.role.clone(),
        pronouns: user_data.pronouns.clone(),
        avatar_url: user_data.avatar_url.clone(),
        banner_url: user_data.banner_url.clone(),
    };

    match repository::update_user(user_id, user_update, &mut conn) {
        Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Error updating user"
        })),
    }
}

pub async fn delete_user(
    uuid: web::Path<String>,
    pool: web::Data<crate::db::Pool>,
) -> impl Responder {
    let user_uuid = uuid.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    // First get the user by UUID
    let user = match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json("User not found"),
    };

    // Then delete the user by ID
    match repository::delete_user(user.id, &mut conn) {
        Ok(count) if count > 0 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().json("User not found"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete user"),
    }
}

// Get user's authentication identities
pub async fn get_user_auth_identities(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate token and get user UUID
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => {
            println!("Successfully validated token for user UUID: {}", claims.sub);
            claims
        },
        Err(e) => {
            println!("Token validation error: {:?}", e);
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid or expired token"
            }));
        },
    };

    // Get user ID from UUID
    println!("Looking up user with UUID: {}", claims.sub);
    let user = match repository::get_user_by_uuid(&claims.sub, &mut conn) {
        Ok(user) => {
            println!("Found user with ID: {}", user.id);
            user
        },
        Err(e) => {
            println!("Error finding user by UUID {}: {:?}", claims.sub, e);
            return HttpResponse::NotFound().json("User not found");
        },
    };

    // Get auth identities for the user
    println!("Fetching auth identities for user ID: {}", user.id);
    match repository::user_auth_identities::get_user_identities_display(user.id, &mut conn) {
        Ok(identities) => {
            println!("Found {} auth identities", identities.len());
            HttpResponse::Ok().json(identities)
        },
        Err(e) => {
            eprintln!("Error fetching auth identities: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve auth identities"
            }))
        }
    }
}

// Get user's authentication identities by UUID
pub async fn get_user_auth_identities_by_uuid(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    path: web::Path<String>, // User UUID
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database connection error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Could not get database connection"
            }));
        }
    };

    let user_uuid = path.into_inner();
    println!("GET auth identities for UUID: {}", user_uuid);

    // Validate token and get user UUID from token
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => {
            println!("Token validated for user: {}", claims.sub);
            claims
        },
        Err(e) => {
            eprintln!("Token validation error: {:?}", e);
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid or expired token"
            }));
        }
    };

    // Ensure the user is authorized (either accessing their own identities or is an admin)
    if claims.sub != user_uuid && claims.role != "admin" {
        eprintln!("Authorization failed: user {} tried to access identities of {}", claims.sub, user_uuid);
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Not authorized to access this resource"
        }));
    }

    // Get auth identities for the user by UUID
    println!("Fetching auth identities for user UUID: {}", user_uuid);
    match repository::user_auth_identities::get_user_identities_display_by_uuid(&user_uuid, &mut conn) {
        Ok(identities) => {
            println!("Found {} auth identities for UUID {}", identities.len(), user_uuid);
            // Log details of each identity
            for (i, identity) in identities.iter().enumerate() {
                println!("  Identity {}: id={}, provider_type={}, email={:?}", 
                         i+1, identity.id, identity.provider_type, identity.email);
            }
            HttpResponse::Ok().json(identities)
        },
        Err(e) => {
            eprintln!("Error fetching auth identities for UUID {}: {:?}", user_uuid, e);
            match e {
                diesel::result::Error::NotFound => {
                    HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "User not found"
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to retrieve auth identities",
                        "error": format!("{:?}", e)
                    }))
                }
            }
        }
    }
}

// Delete a user authentication identity
pub async fn delete_user_auth_identity(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    path: web::Path<i32>, // Auth identity ID
) -> impl Responder {
    let identity_id = path.into_inner();
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Get the user ID
    let user = match repository::get_user_by_uuid(&claims.sub, &mut conn) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Error getting user by UUID: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to get user data"
            }));
        }
    };

    // Ensure the user has at least one other auth method before deleting
    // (to prevent locking themselves out)
    let identities = match repository::user_auth_identities::get_user_identities(user.id, &mut conn) {
        Ok(identities) => identities,
        Err(e) => {
            eprintln!("Error getting user auth identities: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to get authentication identities"
            }));
        }
    };

    if identities.len() <= 1 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Cannot delete the only authentication method. Add another method first."
        }));
    }

    // Delete the identity
    match repository::user_auth_identities::delete_identity(identity_id, user.id, &mut conn) {
        Ok(count) => {
            if count == 0 {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "Authentication identity not found or doesn't belong to you"
                }))
            } else {
                HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": "Authentication identity deleted successfully"
                }))
            }
        },
        Err(e) => {
            eprintln!("Error deleting user auth identity: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to delete authentication identity"
            }))
        }
    }
}

// Delete a user authentication identity by UUID
pub async fn delete_user_auth_identity_by_uuid(
    db_pool: web::Data<crate::db::Pool>,
    auth: BearerAuth,
    path: web::Path<(String, i32)>, // (User UUID, Auth identity ID)
) -> impl Responder {
    let (user_uuid, identity_id) = path.into_inner();
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Ensure the user is authorized (either accessing their own identities or is an admin)
    if claims.sub != user_uuid && claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Not authorized to access this resource"
        }));
    }

    // Ensure the user has at least one other auth method before deleting
    // (to prevent locking themselves out)
    let identities = match repository::user_auth_identities::get_user_identities_by_uuid(&user_uuid, &mut conn) {
        Ok(identities) => identities,
        Err(e) => {
            eprintln!("Error getting user auth identities: {:?}", e);
            return match e {
                diesel::result::Error::NotFound => {
                    HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "User not found"
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to get authentication identities"
                    }))
                }
            };
        }
    };

    if identities.len() <= 1 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Cannot delete the only authentication method. Add another method first."
        }));
    }

    // Delete the identity
    match repository::user_auth_identities::delete_identity_by_uuid(identity_id, &user_uuid, &mut conn) {
        Ok(count) => {
            if count == 0 {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "Authentication identity not found or doesn't belong to this user"
                }))
            } else {
                HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": "Authentication identity deleted successfully"
                }))
            }
        },
        Err(e) => {
            eprintln!("Error deleting user auth identity: {:?}", e);
            match e {
                diesel::result::Error::NotFound => {
                    HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "User or auth identity not found"
                    }))
                },
                _ => {
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to delete authentication identity"
                    }))
                }
            }
        }
    }
}

pub async fn update_user_by_uuid(
    db_pool: web::Data<crate::db::Pool>,
    path: web::Path<String>,
    user_data: web::Json<UserUpdateForm>,
) -> impl Responder {
    let user_uuid = path.into_inner();
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Get the user by UUID first
    let user = match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "User not found"
        })),
    };

    // Check if email is being updated and if it's already in use
    if let Some(email) = &user_data.email {
        if let Ok(existing_user) = repository::get_user_by_email(email, &mut conn) {
            if existing_user.id != user.id {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Email already in use by another user"
                }));
            }
        }
    }

    // Create password hash if password is provided
    if let Some(password) = &user_data.password {
        use bcrypt::hash;
        use crate::models::NewUserAuthIdentity;
        
        // Hash the new password
        let password_hash = match hash(password, DEFAULT_COST) {
            Ok(hash) => hash.into_bytes(),
            Err(_) => return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error hashing password"
            })),
        };
        
        // Find existing local auth identity
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
                    crate::schema::user_auth_identities::table.filter(crate::schema::user_auth_identities::id.eq(identity.id))
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
                let new_auth_identity = NewUserAuthIdentity {
                    user_id: user.id,
                    auth_provider_id: identity.auth_provider_id,
                    provider_user_id: identity.provider_user_id.clone(),
                    email: identity.email.clone(),
                    identity_data: identity.identity_data.clone(),
                    password_hash: Some(password_hash),
                };
                
                if let Err(e) = repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
                    eprintln!("Error creating updated auth identity: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error updating password"
                    }));
                }
            },
            None => {
                // Create new local identity if none exists
                let new_auth_identity = NewUserAuthIdentity {
                    user_id: user.id,
                    auth_provider_id: 1, // Local provider
                    provider_user_id: Uuid::new_v4().to_string(), // Generate a new provider user ID
                    email: user_data.email.clone(),
                    identity_data: None,
                    password_hash: Some(password_hash),
                };
                
                if let Err(e) = repository::user_auth_identities::create_identity(new_auth_identity, &mut conn) {
                    eprintln!("Error creating auth identity: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error setting password"
                    }));
                }
            }
        }
    }

    // Update user
    let user_update = UserUpdate {
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        role: user_data.role.clone(),
        pronouns: user_data.pronouns.clone(),
        avatar_url: user_data.avatar_url.clone(),
        banner_url: user_data.banner_url.clone(),
    };

    match repository::update_user(user.id, user_update, &mut conn) {
        Ok(updated_user) => {
            println!("User updated successfully: {:?}", updated_user);
            HttpResponse::Ok().json(UserResponse::from(updated_user))
        },
        Err(e) => {
            eprintln!("Error updating user: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error updating user"
            }))
        },
    }
}

// Upload user profile images (avatar or banner)
pub async fn upload_user_image(
    uuid: web::Path<String>,
    mut payload: Multipart,
    pool: web::Data<crate::db::Pool>,
    type_query: web::Query<UserImageTypeQuery>,
) -> impl Responder {
    let user_uuid = uuid.into_inner();
    let image_type = &type_query.type_; // "avatar" or "banner"
    
    println!("Received {} upload request for user {}", image_type, user_uuid);
    
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database connection error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database connection error"
            }));
        }
    };
    
    // Validate that the user exists
    let user = match repository::get_user_by_uuid(&user_uuid, &mut conn) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "User not found"
            }));
        }
    };
    
    // Determine the upload directory based on image type
    let storage_path = match image_type.as_str() {
        "avatar" => "uploads/users/avatars",
        "banner" => "uploads/users/banners",
        _ => {
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Invalid image type. Must be 'avatar' or 'banner'"
            }));
        }
    };
    
    // Ensure the directory exists
    if !Path::new(storage_path).exists() {
        println!("Creating directory: {}", storage_path);
        if let Err(e) = fs::create_dir_all(storage_path) {
            eprintln!("Failed to create directory: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to create storage directory"
            }));
        }
    }
    
    // Process the uploaded image
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Get content type
        let content_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_else(|| "application/octet-stream".to_string());
        
        // Validate content type (only allow images)
        if !content_type.starts_with("image/") {
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Only image files are allowed"
            }));
        }
        
        // Extract file extension from content type
        let file_ext = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Unsupported image format. Allowed: JPEG, PNG, GIF, WEBP"
                }));
            }
        };
        
        let unique_filename = format!("{}_{}.{}", user_uuid, Uuid::new_v4(), file_ext);
        let filepath = format!("{}/{}", storage_path, unique_filename);
        
        println!("Saving {} to: {}", image_type, filepath);
        
        // Read file data
        let mut file_data = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error reading chunk: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Error reading uploaded file"
                    }));
                }
            };
            file_data.extend_from_slice(&data);
        }
        
        // Write the file
        if let Err(e) = fs::write(&filepath, &file_data) {
            eprintln!("Error writing file: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error saving uploaded file"
            }));
        }
        
        // Update the user record with the new image URL
        let url = format!("/{}", filepath); // URL will be relative to the server root
        
        let user_update = UserUpdate {
            name: None,
            email: None,
            role: None,
            pronouns: None,
            avatar_url: if image_type == "avatar" { Some(url.clone()) } else { None },
            banner_url: if image_type == "banner" { Some(url.clone()) } else { None },
        };
        
        match repository::update_user(user.id, user_update, &mut conn) {
            Ok(updated_user) => {
                return HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": format!("User {} updated successfully", image_type),
                    "url": url,
                    "user": UserResponse::from(updated_user)
                }));
            },
            Err(e) => {
                eprintln!("Error updating user: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Error updating user record"
                }));
            }
        }
    }
    
    // If we get here, no file was provided
    HttpResponse::BadRequest().json(json!({
        "status": "error",
        "message": "No image file provided"
    }))
}

#[derive(serde::Deserialize)]
pub struct UserImageTypeQuery {
    pub type_: String, // "avatar" or "banner"
} 