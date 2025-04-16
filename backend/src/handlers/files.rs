use actix_web::{web, HttpResponse, Error};
use actix_multipart::{Field, Multipart};
use futures::StreamExt;
use futures::TryStreamExt;
use serde_json::json;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::db::DbConnection;
use crate::models::NewAttachment;
use crate::repository;

// Upload files and store them locally
pub async fn upload_files(
    mut payload: Multipart,
    pool: web::Data<crate::db::Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    println!("Received file upload request");
    
    let mut conn = pool.get().map_err(|e| {
        eprintln!("Database connection error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    // Create uploads directory if it doesn't exist
    let storage_path = "uploads";
    if !Path::new(storage_path).exists() {
        println!("Creating uploads directory: {}", storage_path);
        fs::create_dir_all(storage_path).map_err(|e| {
            eprintln!("Failed to create uploads directory: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to create uploads directory")
        })?;
    }
    
    // Create a temporary directory for uploads that haven't been associated with a ticket yet
    let temp_path = "uploads/temp";
    if !Path::new(temp_path).exists() {
        println!("Creating temporary uploads directory: {}", temp_path);
        fs::create_dir_all(temp_path).map_err(|e| {
            eprintln!("Failed to create temporary uploads directory: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to create temporary uploads directory")
        })?;
    }

    let mut uploaded_attachments = Vec::new();

    // Process each field in the multipart form
    while let Some(mut field) = payload.try_next().await? {
        // Check if the field name is "files"
        let field_name = field.name();
        if field_name != "files" {
            println!("Skipping non-file field: {}", field_name);
            continue;
        }
        
        // Get the filename from the field
        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), |f| f.to_string());
        
        // Get content type if available
        let content_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_else(|| "application/octet-stream".to_string());
        
        println!("Processing uploaded file: {} ({})", filename, content_type);
        
        // Generate a unique filename to prevent collisions
        let unique_filename = format!("{}_{}", Uuid::new_v4(), filename);
        let filepath = format!("{}/{}", temp_path, unique_filename);
        
        println!("Saving file to: {}", filepath);
        
        // Create a file to save the uploaded content
        let file_path = filepath.clone();
        let _file = web::block(move || std::fs::File::create(file_path))
            .await
            .map_err(|e| {
                eprintln!("Failed to create file: {:?}", e);
                actix_web::error::ErrorInternalServerError("Failed to create file")
            })?;
        
        // Write the field data to the file
        let mut file_data = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| {
                eprintln!("Error reading chunk: {:?}", e);
                actix_web::error::ErrorInternalServerError("Error reading chunk")
            })?;
            file_data.extend_from_slice(&data);
        }
        
        println!("Read {} bytes of data for file {}", file_data.len(), filename);
        
        // Write the collected data to the file
        web::block(move || std::fs::write(filepath, &file_data))
            .await
            .map_err(|e| {
                eprintln!("Error writing to file: {:?}", e);
                actix_web::error::ErrorInternalServerError("Error writing to file")
            })?;
        
        // Create a new attachment record in the database
        let new_attachment = NewAttachment {
            url: format!("/uploads/temp/{}", unique_filename),
            name: filename.clone(),
            comment_id: None, // Not linked to a comment yet
        };
        
        println!("Creating attachment record in database: {:?}", new_attachment);
        
        // Save the attachment to the database
        match repository::create_attachment(&mut conn, new_attachment) {
            Ok(attachment) => {
                let attachment_json = json!({
                    "id": attachment.id,
                    "url": attachment.url,
                    "name": attachment.name
                });
                println!("Attachment created successfully: {:?}", attachment_json);
                uploaded_attachments.push(attachment_json);
            },
            Err(e) => {
                eprintln!("Error creating attachment record: {:?}", e);
                return Err(actix_web::error::ErrorInternalServerError("Error creating attachment record"));
            }
        }
    }
    
    println!("Upload complete. Returning {} attachments", uploaded_attachments.len());
    Ok(HttpResponse::Ok().json(uploaded_attachments))
} 