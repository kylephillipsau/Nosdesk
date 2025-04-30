use actix_web::{web, HttpResponse, Responder};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

use crate::db;
use crate::models::{NewTicket, TicketStatus, TicketPriority, TicketUpdate, ArticleContentChunk, TicketsJson, TicketJson};
use crate::repository;

// Pagination query parameters
#[derive(Deserialize)]
pub struct PaginationParams {
    page: Option<i64>,
    #[serde(rename = "pageSize")]
    page_size: Option<i64>,
    #[serde(rename = "sortField")]
    sort_field: Option<String>,
    #[serde(rename = "sortDirection")]
    sort_direction: Option<String>,
    search: Option<String>,
    status: Option<String>,
    priority: Option<String>,
}

// Paginated response
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    data: Vec<T>,
    total: i64,
    page: i64,
    #[serde(rename = "pageSize")]
    page_size: i64,
    #[serde(rename = "totalPages")]
    total_pages: i64,
}

// Get all tickets
pub async fn get_tickets(pool: web::Data<crate::db::Pool>) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::get_all_tickets(&mut conn) {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(_) => HttpResponse::InternalServerError().json("Failed to get tickets"),
    }
}

// Get paginated tickets
pub async fn get_paginated_tickets(
    pool: web::Data<crate::db::Pool>,
    query: web::Query<PaginationParams>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    // Extract and validate pagination parameters
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);

    match repository::get_paginated_tickets(
        &mut conn,
        page,
        page_size,
        query.sort_field.clone(),
        query.sort_direction.clone(),
        query.search.clone(),
        query.status.clone(),
        query.priority.clone(),
    ) {
        Ok((tickets, total)) => {
            // Calculate total pages
            let total_pages = (total as f64 / page_size as f64).ceil() as i64;
            
            // Create paginated response
            let response = PaginatedResponse {
                data: tickets,
                total,
                page,
                page_size,
                total_pages,
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            eprintln!("Error fetching paginated tickets: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to get paginated tickets")
        },
    }
}

// Get a ticket by ID with comments and related info
pub async fn get_ticket(
    pool: web::Data<crate::db::Pool>,
    params: web::Path<i32>,
) -> impl Responder {
    let ticket_id = params.into_inner();
    
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::get_complete_ticket(&mut conn, ticket_id) {
        Ok(ticket) => {
            HttpResponse::Ok().json(ticket)
        },
        Err(_) => HttpResponse::NotFound().json("Ticket not found"),
    }
}

// Create a new ticket
pub async fn create_ticket(
    pool: web::Data<crate::db::Pool>,
    ticket: web::Json<NewTicket>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::create_ticket(&mut conn, ticket.into_inner()) {
        Ok(ticket) => HttpResponse::Created().json(ticket),
        Err(_) => HttpResponse::InternalServerError().json("Failed to create ticket"),
    }
}

// Update a ticket
pub async fn update_ticket(
    pool: web::Data<crate::db::Pool>,
    path: web::Path<i32>,
    ticket: web::Json<NewTicket>,
) -> impl Responder {
    let ticket_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::update_ticket(&mut conn, ticket_id, ticket.into_inner()) {
        Ok(ticket) => HttpResponse::Ok().json(ticket),
        Err(e) => {
            // Just return a generic error - we can't easily check for NotFound without downcast_ref
            HttpResponse::InternalServerError().json(format!("Failed to update ticket: {}", e))
        },
    }
}

// Delete a ticket
pub async fn delete_ticket(
    pool: web::Data<crate::db::Pool>,
    path: web::Path<i32>,
) -> impl Responder {
    let ticket_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::delete_ticket(&mut conn, ticket_id) {
        Ok(0) => HttpResponse::NotFound().json("Ticket not found"),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().json("Failed to delete ticket"),
    }
}

// Import tickets from JSON file
pub async fn import_tickets_from_json(
    pool: web::Data<crate::db::Pool>,
    json_path: web::Path<String>,
) -> impl Responder {
    let json_path_str = json_path.into_inner();
    let path = Path::new(&json_path_str);
    
    let json_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": format!("Failed to read file: {}", e) }));
        }
    };

    // Parse the JSON
    let tickets_json: TicketsJson = match serde_json::from_str(&json_content) {
        Ok(tickets) => tickets,
        Err(_) => return HttpResponse::BadRequest().json("Failed to parse JSON"),
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    // Import each ticket
    let mut imported_count = 0;
    let mut failed_count = 0;

    for ticket_json in tickets_json.tickets {
        match repository::import_ticket_from_json(&mut conn, &ticket_json) {
            Ok(_) => imported_count += 1,
            Err(_) => failed_count += 1,
        }
    }

    HttpResponse::Ok().json(json!({
        "imported": imported_count,
        "failed": failed_count
    }))
}

// Import tickets from JSON string
pub async fn import_tickets_from_json_string(
    pool: web::Data<crate::db::Pool>,
    tickets_json: web::Json<TicketsJson>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    // Import each ticket
    let mut imported_count = 0;
    let mut failed_count = 0;

    for ticket_json in tickets_json.tickets.iter() {
        match repository::import_ticket_from_json(&mut conn, ticket_json) {
            Ok(_) => imported_count += 1,
            Err(_) => failed_count += 1,
        }
    }

    HttpResponse::Ok().json(json!({
        "imported": imported_count,
        "failed": failed_count
    }))
}

// Create an empty ticket with default values
pub async fn create_empty_ticket(
    pool: web::Data<crate::db::Pool>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    // Create a new ticket with default values
    let now = Local::now().naive_local();
    let empty_ticket = NewTicket {
        title: "New Ticket".to_string(),
        status: TicketStatus::Open,
        priority: TicketPriority::Medium,
        created: now,
        modified: now,
        assignee: None,
        requester: "".to_string(),
        closed_at: None,
    };

    // Create the ticket and then add empty article content
    match repository::create_ticket(&mut conn, empty_ticket) {
        Ok(ticket) => {
            // Create empty article content for the ticket
            let new_article_content = crate::models::NewArticleContent {
                content: Vec::new(), // Empty byte vector for binary content
                ticket_id: ticket.id,
            };
            
            match repository::create_article_content(&mut conn, new_article_content) {
                Ok(_) => {
                    // Return the complete ticket with article content
                    match repository::get_complete_ticket(&mut conn, ticket.id) {
                        Ok(complete_ticket) => HttpResponse::Created().json(complete_ticket),
                        Err(_) => HttpResponse::Created().json(ticket), // Fallback to just the ticket if getting complete ticket fails
                    }
                },
                Err(_) => {
                    // If article content creation fails, still return the ticket
                    HttpResponse::Created().json(ticket)
                }
            }
        },
        Err(e) => {
            println!("Error creating empty ticket: {:?}", e);
            HttpResponse::InternalServerError().json(format!("Failed to create empty ticket: {}", e))
        }
    }
}

// Update ticket partially
pub async fn update_ticket_partial(
    pool: web::Data<crate::db::Pool>,
    path: web::Path<i32>,
    ticket_update: web::Json<TicketUpdate>,
) -> impl Responder {
    let ticket_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    println!("Received update request for ticket {}: {:?}", ticket_id, ticket_update);
    
    // Ensure modified date is set if not provided
    let mut update_data = ticket_update.into_inner();
    if update_data.modified.is_none() {
        let now = Local::now().naive_local();
        println!("No modified date provided, using current time: {}", now);
        update_data.modified = Some(now);
    } else {
        println!("Using provided modified date: {:?}", update_data.modified);
    }

    // If status is being changed to closed, set closed_at timestamp
    if let Some(status) = &update_data.status {
        if *status == TicketStatus::Closed {
            // Check if we're changing to closed status
            let current_ticket = repository::get_ticket_by_id(&mut conn, ticket_id);
            if let Ok(ticket) = current_ticket {
                if ticket.status != TicketStatus::Closed {
                    // Only set closed_at if it's a transition to closed
                    let now = Local::now().naive_local();
                    println!("Setting closed_at to current time: {}", now);
                    update_data.closed_at = Some(Some(now));
                }
            }
        } else if *status != TicketStatus::Closed {
            // If changing from closed to another status, clear the closed_at timestamp
            update_data.closed_at = Some(None);
        }
    }

    match repository::update_ticket_partial(&mut conn, ticket_id, update_data) {
        Ok(ticket) => {
            println!("Successfully updated ticket {}: {:?}", ticket_id, ticket);
            
            // Return the complete ticket with all related data
            match repository::get_complete_ticket(&mut conn, ticket_id) {
                Ok(complete_ticket) => HttpResponse::Ok().json(complete_ticket),
                Err(_) => HttpResponse::Ok().json(ticket), // Fallback to just the ticket
            }
        },
        Err(e) => {
            println!("Error updating ticket {}: {:?}", ticket_id, e);
            HttpResponse::InternalServerError().json(format!("Error updating ticket: {}", e))
        }
    }
}

// Link tickets
pub async fn link_tickets(
    pool: web::Data<crate::db::Pool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (ticket_id, linked_ticket_id) = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::link_tickets(&mut conn, ticket_id, linked_ticket_id) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => {
            println!("Error linking tickets: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to link tickets")
        }
    }
}

// Unlink tickets
pub async fn unlink_tickets(
    pool: web::Data<crate::db::Pool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (ticket_id, linked_ticket_id) = path.into_inner();
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    match repository::unlink_tickets(&mut conn, ticket_id, linked_ticket_id) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => {
            println!("Error unlinking tickets: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to unlink tickets")
        }
    }
} 