use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use serde_json::json;
use uuid::Uuid;

use crate::db::DbConnection;
use crate::models::{NewUser, UserResponse, UserUpdate, UserUpdateForm};
use crate::repository;

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
        uuid,
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        role: user_data.role.clone(),
        password_hash: hash("changeme", DEFAULT_COST).unwrap().into_bytes(), // Default password
    };

    println!("Creating user with name {}", new_user.name);
    match repository::create_user(new_user, &mut conn) {
        Ok(user) => {
            println!("User created successfully");
            HttpResponse::Created().json(UserResponse::from(user))
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
    let password_hash = if let Some(password) = &user_data.password {
        match hash(password, DEFAULT_COST) {
            Ok(hash) => Some(hash.into_bytes()),
            Err(_) => return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Error hashing password"
            })),
        }
    } else {
        None
    };

    // Update user
    let user_update = UserUpdate {
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        role: user_data.role.clone(),
        password_hash,
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
    id: web::Path<i32>,
    pool: web::Data<crate::db::Pool>,
) -> impl Responder {
    let user_id = id.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::delete_user(user_id, &mut conn) {
        Ok(count) if count > 0 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().json("User not found"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete user"),
    }
} 