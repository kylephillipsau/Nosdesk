use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use diesel::result::Error;

use crate::db::Pool;
use crate::models::NewDevice;
use crate::repository;

/// Get all devices
pub async fn get_all_devices(pool: web::Data<Pool>) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::get_all_devices(&mut conn) {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(e) => {
            eprintln!("Database error getting all devices: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to get devices")
        }
    }
}

/// Get a device by ID
pub async fn get_device(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> impl Responder {
    let device_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::get_device_by_id(&mut conn, device_id) {
        Ok(device) => HttpResponse::Ok().json(device),
        Err(e) => match e {
            Error::NotFound => HttpResponse::NotFound().json(format!("Device {} not found", device_id)),
            _ => {
                eprintln!("Database error getting device {}: {:?}", device_id, e);
                HttpResponse::InternalServerError().json(format!("Failed to get device {}", device_id))
            }
        },
    }
}

/// Get a device by ticket ID
pub async fn get_device_by_ticket_id(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> impl Responder {
    let ticket_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::get_device_by_ticket_id(&mut conn, ticket_id) {
        Ok(device) => HttpResponse::Ok().json(device),
        Err(e) => {
            eprintln!("Database error getting device for ticket {}: {:?}", ticket_id, e);
            HttpResponse::InternalServerError().json(format!("Failed to get device for ticket {}", ticket_id))
        }
    }
}

/// Create a new device
pub async fn create_device(
    pool: web::Data<Pool>,
    device: web::Json<NewDevice>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::create_device(&mut conn, device.into_inner()) {
        Ok(device) => HttpResponse::Created().json(device),
        Err(e) => {
            eprintln!("Database error creating device: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to create device")
        }
    }
}

/// Update a device
pub async fn update_device(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
    device: web::Json<NewDevice>,
) -> impl Responder {
    let device_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::update_device(&mut conn, device_id, device.into_inner()) {
        Ok(device) => HttpResponse::Ok().json(device),
        Err(e) => {
            match e {
                Error::NotFound => HttpResponse::NotFound().json(format!("Device {} not found", device_id)),
                _ => {
                    eprintln!("Database error updating device {}: {:?}", device_id, e);
                    HttpResponse::InternalServerError().json(format!("Failed to update device {}", device_id))
                }
            }
        }
    }
}

/// Delete a device
pub async fn delete_device(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> impl Responder {
    let device_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::delete_device(&mut conn, device_id) {
        Ok(count) => {
            if count > 0 {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound().json(format!("Device with ID {} not found", device_id))
            }
        },
        Err(e) => {
            eprintln!("Database error deleting device {}: {:?}", device_id, e);
            HttpResponse::InternalServerError().json(format!("Failed to delete device {}", device_id))
        }
    }
} 