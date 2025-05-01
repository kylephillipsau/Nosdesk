mod db;
mod handlers;
mod models;
mod repository;
mod schema;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Helpdesk API is running!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    // Validate that JWT_SECRET is set
    if std::env::var("JWT_SECRET").is_err() {
        eprintln!("\n========== SECURITY CONFIGURATION ERROR ==========");
        eprintln!("ERROR: JWT_SECRET environment variable must be set");
        eprintln!("Generate a secure key with: openssl rand -base64 32");
        eprintln!("Add it to your .env file or environment variables");
        eprintln!("See README-SECURITY.md for more information");
        eprintln!("================================================\n");
        std::process::exit(1);
    } else {
        // Log that we successfully loaded the JWT_SECRET
        // Don't log the actual secret!
        println!("JWT_SECRET environment variable is set and loaded");
    }
    
    let host = "0.0.0.0";
    let port = env::var("PORT").unwrap_or("8080".to_string()).parse::<u16>().unwrap();

    // Set up database connection pool
    let pool = db::establish_connection_pool();

    // Create uploads directory structure if it doesn't exist
    std::fs::create_dir_all("uploads").unwrap_or_else(|e| {
        eprintln!("Warning: Failed to create uploads directory: {}", e);
    });
    
    std::fs::create_dir_all("uploads/temp").unwrap_or_else(|e| {
        eprintln!("Warning: Failed to create temp uploads directory: {}", e);
    });
    
    std::fs::create_dir_all("uploads/tickets").unwrap_or_else(|e| {
        eprintln!("Warning: Failed to create tickets uploads directory: {}", e);
    });

    // Initialize WebSocket app state for collaborative editing
    let yjs_app_state = web::Data::new(handlers::collaboration::YjsAppState::new(web::Data::new(pool.clone())));

    println!("Starting server at http://{}:{}", host, port);
    println!("You can access the server at:");
    println!("  - http://localhost:{}", port);
    println!("  - http://127.0.0.1:{}", port);
    
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .expose_headers(vec!["content-disposition"])
            .supports_credentials()
            .max_age(3600);

        // Configure JSON payload limits for file uploads
        let json_config = web::JsonConfig::default()
            .limit(50 * 1024 * 1024); // 50MB

        // Configure multipart form limits for file uploads
        let multipart_config = web::FormConfig::default()
            .limit(50 * 1024 * 1024); // 50MB

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(yjs_app_state.clone())
            .app_data(json_config)
            .app_data(multipart_config)
            .route("/health", web::get().to(health_check))
            // Serve static files from the uploads directory
            .service(Files::new("/uploads", "./uploads").show_files_listing())
            .service(
                web::scope("/api")
                    // Authentication endpoints
                    .route("/auth/login", web::post().to(handlers::login))
                    .route("/auth/register", web::post().to(handlers::register))
                    .route("/auth/change-password", web::post().to(handlers::change_password))
                    .route("/auth/me", web::get().to(handlers::get_current_user))
                    
                    // Authentication Provider endpoints
                    .route("/auth/providers", web::get().to(handlers::get_auth_providers))
                    .route("/auth/providers/enabled", web::get().to(handlers::get_enabled_auth_providers))
                    .route("/auth/providers", web::post().to(handlers::create_auth_provider))
                    .route("/auth/providers/{id}", web::get().to(handlers::get_auth_provider))
                    .route("/auth/providers/{id}", web::put().to(handlers::update_auth_provider))
                    .route("/auth/providers/{id}", web::delete().to(handlers::delete_auth_provider))
                    .route("/auth/providers/config", web::post().to(handlers::update_auth_provider_config))
                    .route("/auth/oauth/authorize", web::post().to(handlers::oauth_authorize))
                    .route("/auth/oauth/connect", web::post().to(handlers::oauth_connect))
                    .route("/auth/oauth/logout", web::post().to(handlers::oauth_logout))
                    // For Microsoft OAuth Callback
                    .route("/auth/microsoft/callback", web::get().to(handlers::oauth_callback))
                    
                    // File upload endpoint
                    .route("/upload", web::post().to(handlers::upload_files))
                    
                    // ===== TICKET MANAGEMENT =====
                    // Core ticket operations
                    .route("/tickets", web::get().to(handlers::get_tickets))
                    .route("/tickets/paginated", web::get().to(handlers::get_paginated_tickets))
                    .route("/tickets", web::post().to(handlers::create_ticket))
                    .route("/tickets/empty", web::post().to(handlers::create_empty_ticket))
                    .route("/tickets/{id}", web::get().to(handlers::get_ticket))
                    .route("/tickets/{id}", web::put().to(handlers::update_ticket))
                    .route("/tickets/{id}", web::patch().to(handlers::update_ticket_partial))
                    .route("/tickets/{id}", web::delete().to(handlers::delete_ticket))
                    
                    // Ticket import functionality
                    .route("/import/file", web::post().to(handlers::import_tickets_from_json))
                    .route("/import/json", web::post().to(handlers::import_tickets_from_json_string))
                    
                    // Ticket linking
                    .route("/tickets/{ticket_id}/link/{linked_ticket_id}", web::post().to(handlers::link_tickets))
                    .route("/tickets/{ticket_id}/unlink/{linked_ticket_id}", web::delete().to(handlers::unlink_tickets))
                    
                    // Ticket comments and attachments
                    .route("/tickets/{ticket_id}/comments", web::get().to(handlers::get_comments_by_ticket_id))
                    .route("/tickets/{ticket_id}/comments", web::post().to(handlers::add_comment_to_ticket))
                    .route("/comments/{id}", web::delete().to(handlers::delete_comment))
                    .route("/comments/{comment_id}/attachments", web::post().to(handlers::add_attachment_to_comment))
                    .route("/attachments/{id}", web::delete().to(handlers::delete_attachment))
                    
                    // Collaboration endpoints
                    .service(
                        web::scope("/collaboration")
                            .configure(handlers::collaboration::config)
                    )
                    
                    // ===== PROJECT MANAGEMENT =====
                    .route("/projects", web::get().to(handlers::get_all_projects))
                    .route("/projects", web::post().to(handlers::create_project))
                    .route("/projects/{id}", web::get().to(handlers::get_project))
                    .route("/projects/{id}", web::put().to(handlers::update_project))
                    .route("/projects/{id}", web::delete().to(handlers::delete_project))
                    .route("/projects/{id}/tickets", web::get().to(handlers::get_project_tickets))
                    .route("/projects/{project_id}/tickets/{ticket_id}", web::post().to(handlers::add_ticket_to_project))
                    .route("/projects/{project_id}/tickets/{ticket_id}", web::delete().to(handlers::remove_ticket_from_project))
                    
                    // ===== USER MANAGEMENT =====
                    .route("/users", web::get().to(handlers::get_users))
                    .route("/users", web::post().to(handlers::create_user))
                    .route("/users/{uuid}", web::get().to(handlers::get_user_by_uuid))
                    .route("/users/{uuid}", web::put().to(handlers::update_user_by_uuid))
                    .route("/users/{uuid}", web::delete().to(handlers::delete_user))
                    .route("/users/{uuid}/image", web::post().to(handlers::upload_user_image))
                    .route("/users/auth-identities", web::get().to(handlers::get_user_auth_identities))
                    .route("/users/auth-identities/{id}", web::delete().to(handlers::delete_user_auth_identity))
                    // New routes using UUIDs for auth identities
                    .route("/users/{uuid}/auth-identities", web::get().to(handlers::get_user_auth_identities_by_uuid))
                    .route("/users/{uuid}/auth-identities/{id}", web::delete().to(handlers::delete_user_auth_identity_by_uuid))
                    
                    // ===== DEVICE MANAGEMENT =====
                    .route("/devices", web::get().to(handlers::get_all_devices))
                    .route("/devices", web::post().to(handlers::create_device))
                    .route("/devices/{id}", web::get().to(handlers::get_device))
                    .route("/devices/{id}", web::put().to(handlers::update_device))
                    .route("/devices/{id}", web::delete().to(handlers::delete_device))
                    .route("/tickets/{ticket_id}/device", web::get().to(handlers::get_device_by_ticket_id))
                    
                    // ===== DOCUMENTATION SYSTEM =====
                    // Core documentation operations
                    .route("/documentation/pages", web::get().to(handlers::get_documentation_pages))
                    .route("/documentation/pages", web::post().to(handlers::create_documentation_page))
                    .route("/documentation/pages/{id}", web::get().to(handlers::get_documentation_page))
                    .route("/documentation/pages/{id}", web::put().to(handlers::update_documentation_page))
                    .route("/documentation/pages/{id}", web::delete().to(handlers::delete_documentation_page))
                    
                    // Documentation navigation and hierarchy
                    .route("/documentation/pages/top-level", web::get().to(handlers::get_top_level_documentation_pages))
                    .route("/documentation/pages/parent/{parent_id}", web::get().to(handlers::get_documentation_pages_by_parent_id))
                    .route("/documentation/pages/slug/{slug}", web::get().to(handlers::get_documentation_page_by_slug))
                    .route("/documentation/pages/slug/{slug}/with-children", web::get().to(handlers::get_documentation_page_by_slug_with_children))
                    .route("/documentation/pages/{id}/with-children-by-parent", web::get().to(handlers::get_page_with_children_by_parent_id))
                    
                    // Documentation ordering and structure management
                    .route("/documentation/pages/ordered/top-level", web::get().to(handlers::get_ordered_top_level_pages))
                    .route("/documentation/pages/ordered/parent/{parent_id}", web::get().to(handlers::get_ordered_pages_by_parent_id))
                    .route("/documentation/pages/{id}/with-ordered-children", web::get().to(handlers::get_page_with_ordered_children))
                    .route("/documentation/pages/reorder", web::post().to(handlers::reorder_pages))
                    .route("/documentation/pages/move", web::post().to(handlers::move_page_to_parent))
                    
                    // Ticket-to-documentation integration
                    .route("/tickets/{ticket_id}/documentation", web::get().to(handlers::get_documentation_pages_by_ticket_id))
                    .route("/tickets/{ticket_id}/documentation/create", web::post().to(handlers::create_documentation_page_from_ticket))
                    .route("/documentation/{id}", web::put().to(handlers::update_documentation_page))
                    .route("/documentation/{id}", web::delete().to(handlers::delete_documentation_page))
            )
    })
    .bind((host, port))?
    .run()
    .await
}