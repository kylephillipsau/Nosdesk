use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use diesel::result::Error;

use crate::db::Pool;
use crate::models::{NewProject, ProjectUpdate};
use crate::repository;

// Get all projects with ticket counts
pub async fn get_all_projects(
    pool: web::Data<Pool>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::get_projects_with_ticket_count(&mut conn) {
        Ok(projects) => HttpResponse::Ok().json(projects),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get projects"),
    }
}

// Get a single project by ID with ticket count
pub async fn get_project(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> impl Responder {
    let project_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::get_project_with_ticket_count(&mut conn, project_id) {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => {
            match e {
                Error::NotFound => HttpResponse::NotFound().json("Project not found"),
                _ => HttpResponse::InternalServerError().json("Failed to get project"),
            }
        }
    }
}

// Create a new project
pub async fn create_project(
    pool: web::Data<Pool>,
    project: web::Json<NewProject>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::create_project(&mut conn, project.into_inner()) {
        Ok(project) => HttpResponse::Created().json(project),
        Err(_) => HttpResponse::InternalServerError().json("Failed to create project"),
    }
}

// Update an existing project
pub async fn update_project(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
    project_update: web::Json<ProjectUpdate>,
) -> impl Responder {
    let project_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::update_project(&mut conn, project_id, project_update.into_inner()) {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => {
            match e {
                Error::NotFound => HttpResponse::NotFound().json("Project not found"),
                _ => HttpResponse::InternalServerError().json("Failed to update project"),
            }
        }
    }
}

// Delete a project
pub async fn delete_project(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> impl Responder {
    let project_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::delete_project(&mut conn, project_id) {
        Ok(0) => HttpResponse::NotFound().json("Project not found"),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete project"),
    }
}

// Get all tickets in a project
pub async fn get_project_tickets(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
) -> impl Responder {
    let project_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::get_project_tickets(&mut conn, project_id) {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get project tickets"),
    }
}

// Add a ticket to a project
pub async fn add_ticket_to_project(
    pool: web::Data<Pool>,
    path: web::Path<(i32, i32)>,
    sse_state: web::Data<crate::handlers::sse::SseState>,
) -> impl Responder {
    let (project_id, ticket_id) = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::add_ticket_to_project(&mut conn, project_id, ticket_id) {
        Ok(association) => {
            // Broadcast SSE event for project assignment
            println!("Broadcasting SSE event: Ticket {} assigned to project {}", ticket_id, project_id);
            use crate::utils::sse::SseBroadcaster;
            SseBroadcaster::broadcast_project_assigned(&sse_state, ticket_id, project_id).await;
            
            HttpResponse::Created().json(association)
        },
        Err(_) => HttpResponse::InternalServerError().json("Failed to add ticket to project"),
    }
}

// Remove a ticket from a project
pub async fn remove_ticket_from_project(
    pool: web::Data<Pool>,
    path: web::Path<(i32, i32)>,
    sse_state: web::Data<crate::handlers::sse::SseState>,
) -> impl Responder {
    let (project_id, ticket_id) = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::remove_ticket_from_project(&mut conn, project_id, ticket_id) {
        Ok(0) => HttpResponse::NotFound().json("Association not found"),
        Ok(_) => {
            // Broadcast SSE event for project unassignment
            println!("Broadcasting SSE event: Ticket {} unassigned from project {}", ticket_id, project_id);
            use crate::utils::sse::SseBroadcaster;
            SseBroadcaster::broadcast_project_unassigned(&sse_state, ticket_id, project_id).await;
            
            HttpResponse::NoContent().finish()
        },
        Err(_) => HttpResponse::InternalServerError().json("Failed to remove ticket from project"),
    }
} 