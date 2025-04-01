use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::repository;
use crate::db;
use crate::models::NewArticleContent;

#[derive(Debug, Serialize, Deserialize)]
pub struct CollaborativeUpdate {
    pub doc_id: String,
    pub content: String,
}

// Simple handler to sync ticket article content
pub async fn sync_ticket_article(
    pool: web::Data<crate::db::Pool>,
    update: web::Json<CollaborativeUpdate>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    let doc_id = update.doc_id.clone();
    println!("Syncing content for document: {}", doc_id);
    
    let ticket_id = match doc_id.strip_prefix("ticket-").and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            println!("Invalid ticket ID format: {}", doc_id);
            return HttpResponse::BadRequest().json("Invalid ticket ID format");
        }
    };
    
    let new_article_content = NewArticleContent {
        content: update.content.clone(),
        ticket_id,
    };

    match repository::update_article_content(&mut conn, ticket_id, new_article_content) {
        Ok(article) => {
            println!("Successfully saved article for ticket {}", ticket_id);
            HttpResponse::Ok().json(json!({ 
                "status": "success", 
                "message": "Article synchronized", 
                "article_id": article.id 
            }))
        },
        Err(e) => {
            println!("Failed to sync article for ticket {}: {:?}", ticket_id, e);
            HttpResponse::InternalServerError().json(format!("Failed to sync article: {}", e))
        }
    }
}

// Simple handler to get article content by ticket ID
pub async fn get_article_content(
    pool: web::Data<crate::db::Pool>,
    doc_id: web::Path<String>,
) -> impl Responder {
    let doc_id = doc_id.into_inner();
    let clean_doc_id = doc_id.replace("/", "_");
    
    // Extract ticket ID from doc_id (format: "ticket-123")
    let ticket_id = match clean_doc_id.strip_prefix("ticket-").and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            println!("Invalid ticket ID format: {}", clean_doc_id);
            return HttpResponse::BadRequest().json("Invalid ticket ID format");
        }
    };
    
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::get_article_content_by_ticket_id(&mut conn, ticket_id) {
        Ok(article_content) => {
            println!("Retrieved article content for ticket {}", ticket_id);
            HttpResponse::Ok().json(json!({
                "content": article_content.content,
                "ticket_id": ticket_id
            }))
        },
        Err(e) => {
            println!("No article content found for ticket {}: {}", ticket_id, e);
            HttpResponse::Ok().json(json!({
                "content": "",
                "ticket_id": ticket_id
            }))
        }
    }
}

// Configure routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/collaboration")
            .route("/ticket/sync", web::post().to(sync_ticket_article))
            .route("/article/{doc_id}", web::get().to(get_article_content))
    );
}