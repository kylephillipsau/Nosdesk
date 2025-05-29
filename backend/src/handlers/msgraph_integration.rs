use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde_json::json;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;
use reqwest;
use urlencoding;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::time::Duration;
use image::{ImageFormat, DynamicImage};

use crate::db::{Pool, DbConnection};
use crate::handlers::auth::validate_token_internal;
use crate::repository::auth_providers as auth_provider_repo;
use crate::repository::users as user_repo;
use crate::repository::devices as device_repo;
use crate::repository::user_auth_identities as identity_repo;
use crate::repository::user_emails as user_emails_repo;
use crate::repository::sync_history as sync_history_repo;
use crate::config_utils;
use crate::models::{NewUser, NewUserAuthIdentity, User, UserAuthIdentity, NewSyncHistory, SyncHistoryUpdate};

// Global progress tracker with cancellation support
lazy_static::lazy_static! {
    static ref SYNC_PROGRESS: Arc<Mutex<HashMap<String, SyncProgressState>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref SYNC_CANCELLATION: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Configuration constants for optimization
const CONCURRENT_PHOTO_DOWNLOADS: usize = 10; // Number of concurrent photo downloads
const CONCURRENT_USER_PROCESSING: usize = 8; // Number of concurrent user processing tasks
const BATCH_SIZE: usize = 50; // Number of users to process in each batch
const USER_BATCH_SIZE: usize = 25; // Number of users to process in each user sync batch
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30); // HTTP request timeout
const RETRY_ATTEMPTS: usize = 3; // Number of retry attempts for failed requests

// Helper function to get configurable concurrency settings
fn get_concurrency_config() -> (usize, usize) {
    let concurrent_downloads = std::env::var("MSGRAPH_CONCURRENT_DOWNLOADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(CONCURRENT_PHOTO_DOWNLOADS);
    
    let batch_size = std::env::var("MSGRAPH_BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(BATCH_SIZE);
    
    (concurrent_downloads, batch_size)
}

// Helper function to get user sync concurrency settings
fn get_user_sync_config() -> (usize, usize) {
    let concurrent_processing = std::env::var("MSGRAPH_CONCURRENT_USER_PROCESSING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(CONCURRENT_USER_PROCESSING);
    
    let user_batch_size = std::env::var("MSGRAPH_USER_BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(USER_BATCH_SIZE);
    
    (concurrent_processing, user_batch_size)
}

#[derive(Serialize, Debug, Clone)]
pub struct SyncProgressState {
    pub session_id: String,
    pub entity: String,
    pub current: usize,
    pub total: usize,
    pub status: String,
    pub message: String,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sync_type: String, // "users", "profile_photos", "devices", "groups"
}

#[derive(Deserialize, Debug)]
pub struct SyncDataRequest {
    pub entities: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct ConnectionStatus {
    pub status: String,
    pub message: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub available_entities: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct SyncProgress {
    pub entity: String,
    pub processed: usize,
    pub total: usize,
    pub status: String,
    pub errors: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub results: Vec<SyncProgress>,
    pub total_processed: usize,
    pub total_errors: usize,
}

// Microsoft Graph User structure from API response
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MicrosoftGraphUser {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub mail: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: String,
    #[serde(rename = "jobTitle")]
    pub job_title: Option<String>,
    pub department: Option<String>,
    #[serde(rename = "officeLocation")]
    pub office_location: Option<String>,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: Option<String>,
    #[serde(rename = "businessPhones")]
    pub business_phones: Option<Vec<String>>,
    #[serde(rename = "proxyAddresses")]
    pub proxy_addresses: Option<Vec<String>>,
    #[serde(rename = "otherMails")]
    pub other_mails: Option<Vec<String>>,
}

// Microsoft Graph Device structure from API response (managedDevice)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MicrosoftGraphDevice {
    pub id: String,
    #[serde(rename = "deviceName")]
    pub device_name: Option<String>,
    #[serde(rename = "operatingSystem")]
    pub operating_system: Option<String>,
    #[serde(rename = "osVersion")]
    pub os_version: Option<String>,
    #[serde(rename = "manufacturer")]
    pub manufacturer: Option<String>,
    #[serde(rename = "model")]
    pub model: Option<String>,
    #[serde(rename = "serialNumber")]
    pub serial_number: Option<String>,
    #[serde(rename = "azureADDeviceId")]
    pub azure_ad_device_id: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "complianceState")]
    pub compliance_state: Option<String>,
    #[serde(rename = "lastSyncDateTime")]
    pub last_sync_date_time: Option<String>,
    #[serde(rename = "enrolledDateTime")]
    pub enrolled_date_time: Option<String>,
    #[serde(rename = "deviceEnrollmentType")]
    pub device_enrollment_type: Option<String>,
    #[serde(rename = "managementAgent")]
    pub management_agent: Option<String>,
}

// User sync statistics
#[derive(Serialize, Debug)]
pub struct UserSyncStats {
    pub new_users_created: usize,
    pub existing_users_updated: usize,
    pub identities_linked: usize,
    pub errors: Vec<String>,
}

// Device sync statistics
#[derive(Serialize, Debug)]
pub struct DeviceSyncStats {
    pub new_devices_created: usize,
    pub existing_devices_updated: usize,
    pub devices_assigned: usize,
    pub errors: Vec<String>,
}

// Optimized photo sync result
#[derive(Debug)]
struct PhotoSyncResult {
    user_id: i32,
    user_name: String,
    avatar_url: Option<String>,
    success: bool,
    error: Option<String>,
}

// User sync result for concurrent processing
#[derive(Debug)]
struct UserSyncResult {
    user_principal_name: String,
    operation: String, // "created", "updated", "linked", "error"
    success: bool,
    error: Option<String>,
}

// Helper functions for progress tracking
fn update_sync_progress(session_id: &str, entity: &str, current: usize, total: usize, status: &str, message: &str) {
    update_sync_progress_with_type(session_id, entity, current, total, status, message, entity);
}

fn update_sync_progress_with_type(session_id: &str, entity: &str, current: usize, total: usize, status: &str, message: &str, sync_type: &str) {
    let now = Utc::now();
    
    // Update in-memory progress for real-time tracking
    if let Ok(mut progress_map) = SYNC_PROGRESS.lock() {
        let progress = SyncProgressState {
            session_id: session_id.to_string(),
            entity: entity.to_string(),
            current,
            total,
            status: status.to_string(),
            message: message.to_string(),
            started_at: progress_map.get(session_id)
                .map(|p| p.started_at)
                .unwrap_or(now),
            updated_at: now,
            sync_type: sync_type.to_string(),
        };
        progress_map.insert(session_id.to_string(), progress);
    }
    
    // Only persist to database for final states (completed, error, cancelled)
    if status == "completed" || status == "error" || status == "cancelled" {
        persist_final_sync_to_db(session_id, entity, current, total, status, message, sync_type);
    }
}

fn persist_final_sync_to_db(session_id: &str, entity: &str, current: usize, total: usize, status: &str, message: &str, sync_type: &str) {
    let session_id = session_id.to_string();
    let entity = entity.to_string();
    let status = status.to_string();
    let message = message.to_string();
    let sync_type = sync_type.to_string();
    
    let pool = crate::db::establish_connection_pool();
    if let Ok(mut conn) = pool.get() {
        let now = Utc::now().naive_utc();
        
        let update = SyncHistoryUpdate {
            status: Some(status.clone()),
            message: Some(message.clone()),
            current_count: Some(current as i32),
            total_count: Some(total as i32),
            updated_at: Some(now),
            completed_at: Some(Some(now)),
        };
        
        match sync_history_repo::update_sync_history(&mut conn, &session_id, update) {
            Ok(_) => {},
            Err(_) => {
                // If update fails, try to create a new record
                let new_sync = NewSyncHistory {
                    session_id: session_id.clone(),
                    sync_type,
                    entity,
                    status: status.clone(),
                    message,
                    current_count: current as i32,
                    total_count: total as i32,
                    started_at: now,
                    updated_at: now,
                    completed_at: Some(now),
                };
                
                let _ = sync_history_repo::create_sync_history(&mut conn, new_sync);
            }
        }
    }
}

fn get_sync_progress(session_id: &str) -> Option<SyncProgressState> {
    if let Ok(progress_map) = SYNC_PROGRESS.lock() {
        progress_map.get(session_id).cloned()
    } else {
        None
    }
}

fn clear_sync_progress(session_id: &str) {
    if let Ok(mut progress_map) = SYNC_PROGRESS.lock() {
        progress_map.remove(session_id);
    }
    if let Ok(mut cancellation_map) = SYNC_CANCELLATION.lock() {
        cancellation_map.remove(session_id);
    }
}

// Cancellation support functions
fn is_sync_cancelled(session_id: &str) -> bool {
    if let Ok(cancellation_map) = SYNC_CANCELLATION.lock() {
        cancellation_map.get(session_id).copied().unwrap_or(false)
    } else {
        false
    }
}

fn cancel_sync(session_id: &str) {
    if let Ok(mut cancellation_map) = SYNC_CANCELLATION.lock() {
        cancellation_map.insert(session_id.to_string(), true);
    }
}

fn initialize_sync_session(session_id: &str) {
    if let Ok(mut cancellation_map) = SYNC_CANCELLATION.lock() {
        cancellation_map.insert(session_id.to_string(), false);
    }
}

/// Get sync progress for a specific session
pub async fn get_sync_progress_endpoint(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    path: web::Path<String>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    let session_id = path.into_inner();
    
    match get_sync_progress(&session_id) {
        Some(progress) => HttpResponse::Ok().json(progress),
        None => HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Sync session not found"
        }))
    }
}

/// Get all active sync sessions
pub async fn get_active_syncs(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    if let Ok(progress_map) = SYNC_PROGRESS.lock() {
        let active_syncs: Vec<SyncProgressState> = progress_map
            .values()
            .filter(|progress| progress.status == "running" || progress.status == "starting")
            .cloned()
            .collect();
        
        HttpResponse::Ok().json(json!({
            "active_syncs": active_syncs,
            "count": active_syncs.len()
        }))
    } else {
        HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to access sync progress"
        }))
    }
}

/// Get the most recent completed sync session
pub async fn get_last_sync(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Try to get from database first (persistent storage)
    match sync_history_repo::get_last_completed_sync(&mut conn) {
        Ok(sync_history) => {
            // Convert database model to API response format
            let response = SyncProgressState {
                session_id: sync_history.session_id,
                entity: sync_history.entity,
                current: sync_history.current_count as usize,
                total: sync_history.total_count as usize,
                status: sync_history.status,
                message: sync_history.message,
                started_at: DateTime::from_naive_utc_and_offset(sync_history.started_at, Utc),
                updated_at: DateTime::from_naive_utc_and_offset(sync_history.updated_at, Utc),
                sync_type: sync_history.sync_type,
            };
            HttpResponse::Ok().json(response)
        },
        Err(_) => {
            // Fallback to in-memory storage if database query fails
            if let Ok(progress_map) = SYNC_PROGRESS.lock() {
                let last_sync = progress_map
                    .values()
                    .filter(|progress| {
                        progress.status == "completed" || 
                        progress.status == "error" || 
                        progress.status == "cancelled"
                    })
                    .max_by_key(|progress| progress.updated_at);
                
                match last_sync {
                    Some(sync) => HttpResponse::Ok().json(sync),
                    None => HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "No completed synchronizations found"
                    }))
                }
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to access sync progress"
                }))
            }
        }
    }
}

/// Cancel a sync session
pub async fn cancel_sync_session(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    path: web::Path<String>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    let session_id = path.into_inner();
    
    // Check if the session exists and is cancellable
    if let Some(progress) = get_sync_progress(&session_id) {
        if progress.status == "running" || progress.status == "starting" {
            cancel_sync(&session_id);
            update_sync_progress_with_type(&session_id, &progress.entity, progress.current, progress.total, "cancelling", "Cancellation requested", &progress.sync_type);
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Sync cancellation requested"
            }))
        } else {
            HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Sync is not running"
            }))
        }
    } else {
        HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Sync session not found"
        }))
    }
}

/// Get Microsoft Graph connection status
pub async fn get_connection_status(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if Microsoft auth provider exists and is enabled
    let microsoft_provider = match auth_provider_repo::get_default_provider_by_type("microsoft", &mut conn) {
        Ok(provider) => {
            if !provider.enabled {
                return HttpResponse::Ok().json(ConnectionStatus {
                    status: "disconnected".to_string(),
                    message: "Microsoft auth provider is disabled".to_string(),
                    last_sync: None,
                    available_entities: vec![],
                });
            }
            provider
        },
        Err(_) => {
            return HttpResponse::Ok().json(ConnectionStatus {
                status: "disconnected".to_string(),
                message: "Microsoft auth provider not configured".to_string(),
                last_sync: None,
                available_entities: vec![],
            });
        }
    };

    // Check environment configuration
    let config_check = check_microsoft_config();
    if let Err(error_msg) = config_check {
        return HttpResponse::Ok().json(ConnectionStatus {
            status: "error".to_string(),
            message: format!("Configuration error: {}", error_msg),
            last_sync: None,
            available_entities: vec![],
        });
    }

    // If we get here, configuration looks good
    HttpResponse::Ok().json(ConnectionStatus {
        status: "connected".to_string(),
        message: "Microsoft Graph connection is configured and ready".to_string(),
        last_sync: None, // TODO: Track actual sync times
        available_entities: vec!["users".to_string(), "devices".to_string(), "groups".to_string()],
    })
}

/// Test Microsoft Graph connection
pub async fn test_connection(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Get Microsoft provider
    let provider = match auth_provider_repo::get_default_provider_by_type("microsoft", &mut conn) {
        Ok(provider) => provider,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Microsoft auth provider not found"
        })),
    };

    // Test the connection by making a simple Graph API call
    match test_graph_connection(provider.id).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(error) => HttpResponse::Ok().json(json!({
            "success": false,
            "status": "error",
            "message": format!("Connection test failed: {}", error)
        }))
    }
}

/// Sync data from Microsoft Graph
pub async fn sync_data(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    request: web::Json<SyncDataRequest>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Get Microsoft provider
    let provider = match auth_provider_repo::get_default_provider_by_type("microsoft", &mut conn) {
        Ok(provider) => provider,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Microsoft auth provider not found"
        })),
    };

    // Generate a unique session ID for tracking progress
    let session_id = Uuid::new_v4().to_string();

    // Initialize session and progress tracking
    initialize_sync_session(&session_id);

    // Determine the primary sync type based on entities
    let entities = request.entities.clone();
    let primary_sync_type = if entities.contains(&"devices".to_string()) {
        "devices"
    } else if entities.contains(&"users".to_string()) {
        "users"
    } else if entities.contains(&"groups".to_string()) {
        "groups"
    } else {
        "sync"
    };
    
    update_sync_progress_with_type(&session_id, "initializing", 0, 0, "starting", "Initializing sync process", primary_sync_type);

    // Start the sync process in the background
    let provider_id = provider.id;
    let session_id_clone = session_id.clone();
    
    tokio::spawn(async move {
        let mut conn = match db_pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                update_sync_progress(&session_id_clone, "error", 0, 0, "error", "Database connection failed");
                return;
            }
        };

        match perform_sync(&mut conn, provider_id, &entities, &session_id_clone).await {
            Ok(sync_result) => {
                // Check if sync was cancelled by looking at the result
                if !sync_result.success && sync_result.message.contains("cancelled") {
                    // This was a cancellation, not a completion
                    // The individual entity sync functions already updated the progress correctly
                    // No need to overwrite with generic completion message
                } else {
                    // Normal completion - update with final status
                let completion_message = format!("Sync completed successfully: {} items processed", sync_result.total_processed);
                
                    // Don't overwrite entity-specific progress, just ensure database is updated
                let now = Utc::now().naive_utc();
                let update = SyncHistoryUpdate {
                    status: Some("completed".to_string()),
                    message: Some(completion_message),
                    current_count: Some(sync_result.total_processed as i32),
                    total_count: Some(sync_result.total_processed as i32),
                    updated_at: Some(now),
                    completed_at: Some(Some(now)),
                };
                let _ = sync_history_repo::update_sync_history(&mut conn, &session_id_clone, update);
                }
            },
            Err(error) => {
                let error_message = format!("Sync failed: {}", error);
                
                // Determine the primary sync type for error reporting
                let primary_sync_type = if entities.contains(&"devices".to_string()) {
                    "devices"
                } else if entities.contains(&"users".to_string()) {
                    "users"
                } else if entities.contains(&"groups".to_string()) {
                    "groups"
                } else {
                    "sync"
                };
                
                update_sync_progress_with_type(&session_id_clone, primary_sync_type, 0, 0, "error", &error_message, primary_sync_type);
                
                // Also immediately update the database synchronously for errors
                let now = Utc::now().naive_utc();
                let update = SyncHistoryUpdate {
                    status: Some("error".to_string()),
                    message: Some(error_message),
                    current_count: Some(0),
                    total_count: Some(0),
                    updated_at: Some(now),
                    completed_at: Some(Some(now)),
                };
                let _ = sync_history_repo::update_sync_history(&mut conn, &session_id_clone, update);
            }
        }
    });

    // Return the session ID immediately
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Sync started successfully",
        "session_id": session_id
    }))
}





/// Check Microsoft configuration
fn check_microsoft_config() -> Result<(), String> {
    config_utils::get_microsoft_client_id().map_err(|e| format!("Client ID: {}", e))?;
    config_utils::get_microsoft_tenant_id().map_err(|e| format!("Tenant ID: {}", e))?;
    config_utils::get_microsoft_client_secret().map_err(|e| format!("Client Secret: {}", e))?;
    config_utils::get_microsoft_redirect_uri().map_err(|e| format!("Redirect URI: {}", e))?;
    Ok(())
}

/// Test Graph connection by making a simple API call
async fn test_graph_connection(provider_id: i32) -> Result<serde_json::Value, String> {
    // Get required configuration values from environment variables
    let client_id = config_utils::get_microsoft_client_id()
        .map_err(|e| format!("Microsoft Client ID configuration error: {}", e))?;
    
    let tenant_id = config_utils::get_microsoft_tenant_id()
        .map_err(|e| format!("Microsoft Tenant ID configuration error: {}", e))?;
    
    let client_secret = config_utils::get_microsoft_client_secret()
        .map_err(|e| format!("Microsoft Client Secret configuration error: {}", e))?;

    // Get an access token using client credentials flow
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "client_credentials"),
        ("scope", "https://graph.microsoft.com/.default"),
    ];

    let client = reqwest::Client::new();
    let start_time = std::time::Instant::now();
    
    let token_response = client
        .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to request access token: {}", e))?;

    let token_data: serde_json::Value = token_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| {
            let error_desc = token_data.get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            format!("Failed to obtain access token: {}", error_desc)
        })?;

    // Test the connection with a simple API call to get organization info
    let url = "https://graph.microsoft.com/v1.0/organization";
    
    let graph_response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to send Microsoft Graph test request: {}", e))?;

    let response_time = start_time.elapsed().as_millis();
    let status = graph_response.status();
    
    if status.is_success() {
        let response_data: serde_json::Value = graph_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Microsoft Graph response: {}", e))?;

        // Check if we got organization data
        let org_count = response_data
            .get("value")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(json!({
            "success": true,
            "status": "connected",
            "message": "Successfully connected to Microsoft Graph API",
            "details": {
                "test_endpoint": "/organization",
                "response_time_ms": response_time,
                "permissions_verified": true,
                "organization_count": org_count
            }
        }))
    } else {
        let error_data: serde_json::Value = graph_response
            .json()
            .await
            .unwrap_or_else(|_| json!({"error": {"message": "Unknown error"}}));
        
        let error_msg = error_data
            .get("error")
            .and_then(|err| err.get("message"))
            .and_then(|msg| msg.as_str())
            .unwrap_or("Unknown Microsoft Graph error");
            
        Err(format!("Microsoft Graph API error ({}): {}", status, error_msg))
    }
}

/// Perform data synchronization
async fn perform_sync(
    conn: &mut DbConnection,
    provider_id: i32,
    entities: &[String],
    session_id: &str,
) -> Result<SyncResult, String> {
    let mut results = Vec::new();
    let mut total_processed = 0;
    let mut total_errors = 0;
    let mut was_cancelled = false;

    // Determine the primary sync type based on entities
    let primary_sync_type = if entities.contains(&"devices".to_string()) {
        "devices"
    } else if entities.contains(&"users".to_string()) {
        "users"
    } else if entities.contains(&"groups".to_string()) {
        "groups"
    } else {
        "sync"
    };

    // Don't overwrite entity-specific progress - just process each entity
    for (index, entity) in entities.iter().enumerate() {
        let sync_progress = match entity.as_str() {
            "users" => sync_users(conn, provider_id, session_id).await,
            "devices" => sync_devices(conn, provider_id, session_id).await,
            "groups" => sync_groups(conn, provider_id, session_id).await,
            _ => {
                total_errors += 1;
                // Update progress with error for unsupported entity
                update_sync_progress_with_type(session_id, entity, 0, 0, "error", &format!("Unsupported entity type: {}", entity), primary_sync_type);
                SyncProgress {
                    entity: entity.clone(),
                    processed: 0,
                    total: 0,
                    status: "error".to_string(),
                    errors: vec![format!("Unsupported entity type: {}", entity)],
                }
            }
        };

        // Check if sync was cancelled
        if sync_progress.status == "cancelled" {
            was_cancelled = true;
        }

        total_processed += sync_progress.processed;
        total_errors += sync_progress.errors.len();
        results.push(sync_progress);

        // Break early if cancelled
        if was_cancelled {
            break;
        }
    }

    Ok(SyncResult {
        success: total_errors == 0 && !was_cancelled,
        message: if was_cancelled {
            format!("Sync was cancelled. Processed {} items", total_processed)
        } else if total_errors == 0 {
            format!("Successfully synchronized {} items", total_processed)
        } else {
            format!("Synchronized {} items with {} errors", total_processed, total_errors)
        },
        results,
        total_processed,
        total_errors,
    })
}

/// Sync users from Microsoft Graph (optimized with concurrent processing)
async fn sync_users(
    conn: &mut DbConnection,
    provider_id: i32,
    session_id: &str,
) -> SyncProgress {
    let mut stats = UserSyncStats {
        new_users_created: 0,
        existing_users_updated: 0,
        identities_linked: 0,
        errors: Vec::new(),
    };

    update_sync_progress(session_id, "users", 0, 0, "running", "Fetching users from Microsoft Graph");

    // Step 1: Fetch users from Microsoft Graph
    let (microsoft_users, access_token) = match fetch_microsoft_graph_users_optimized(provider_id).await {
        Ok(result) => result,
        Err(error) => {
            update_sync_progress(session_id, "users", 0, 0, "error", &format!("Failed to fetch users: {}", error));
            return SyncProgress {
                entity: "users".to_string(),
                processed: 0,
                total: 0,
                status: "error".to_string(),
                errors: vec![format!("Failed to fetch Microsoft Graph users: {}", error)],
            };
        }
    };

    let total_users = microsoft_users.len();
    println!("Fetched {} users from Microsoft Graph", total_users);

    if total_users == 0 {
        update_sync_progress(session_id, "users", 0, 0, "completed", "No users found to sync");
        return SyncProgress {
            entity: "users".to_string(),
            processed: 0,
            total: 0,
            status: "completed".to_string(),
            errors: Vec::new(),
        };
    }

    update_sync_progress(session_id, "users", 0, total_users, "running", &format!("Processing {} users concurrently", total_users));

    // Get concurrency configuration
    let (concurrent_processing, user_batch_size) = get_user_sync_config();

    // Create a shared HTTP client for profile photo downloads
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .pool_max_idle_per_host(concurrent_processing)
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            let error_msg = format!("Failed to create HTTP client: {}", e);
            update_sync_progress(session_id, "users", 0, total_users, "error", &error_msg);
            return SyncProgress {
                entity: "users".to_string(),
                processed: 0,
                total: total_users,
                status: "error".to_string(),
                errors: vec![error_msg],
            };
        }).unwrap();

    // Step 2: Process users in optimized batches
    let mut processed_count = 0;
    let mut sync_was_cancelled = false;

    // Process users in batches with optimized database operations
    for batch in microsoft_users.chunks(user_batch_size) {
        let batch_start = processed_count;
        let batch_size = batch.len();
        
        update_sync_progress(
            session_id, 
            "users", 
            batch_start, 
            total_users, 
            "running", 
            &format!("Processing batch {}-{} of {}", batch_start + 1, batch_start + batch_size, total_users)
        );

        // Process each user in the batch with optimized profile photo handling
        for ms_user in batch {
            // Check for cancellation before processing each user
            if is_sync_cancelled(session_id) {
                let processed = stats.new_users_created + stats.existing_users_updated + stats.identities_linked;
                let cancel_message = format!("Sync was cancelled by user request. Processed {} of {} users ({} created, {} updated, {} linked)", 
                    processed_count, total_users, stats.new_users_created, stats.existing_users_updated, stats.identities_linked);
                
                // Update progress with cancellation status
                update_sync_progress_with_type(
                    session_id, 
                    "users", 
                    processed_count, 
                    total_users, 
                    "cancelled", 
                    &cancel_message, 
                    "users"
                );
                
                sync_was_cancelled = true;
                return SyncProgress {
                    entity: "users".to_string(),
                    processed,
                    total: total_users,
                    status: "cancelled".to_string(),
                    errors: stats.errors,
                };
            }
            
            processed_count += 1;
            
            update_sync_progress_with_type(
                session_id, 
                "users", 
                processed_count - 1, 
                total_users, 
                "running", 
                &format!("Processing user: {}", ms_user.user_principal_name),
                "users"
            );

            match process_microsoft_user_optimized_v2(conn, provider_id, ms_user, &mut stats, &access_token, &client).await {
                Ok(_) => {
                    println!("Successfully processed user: {}", ms_user.user_principal_name);
                },
                Err(error) => {
                    let error_msg = format!("Failed to process user {}: {}", ms_user.user_principal_name, error);
                    println!("{}", error_msg);
                    stats.errors.push(error_msg);
                }
            }

            // Update progress more frequently
            if processed_count % 5 == 0 || processed_count == total_users {
                let processed = stats.new_users_created + stats.existing_users_updated + stats.identities_linked;
                update_sync_progress(
                    session_id, 
                    "users", 
                    processed_count, 
                    total_users, 
                    "running", 
                    &format!("Processed {}/{} users ({} created, {} updated, {} linked, {} errors)", 
                        processed_count, total_users, stats.new_users_created, stats.existing_users_updated, 
                        stats.identities_linked, stats.errors.len())
                );
            }
        }

        // Small delay between batches to be respectful to the API and database
        if processed_count < total_users {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    let processed = stats.new_users_created + stats.existing_users_updated + stats.identities_linked;

    // Only mark as completed if the sync wasn't cancelled
    if !sync_was_cancelled {
        update_sync_progress(
            session_id, 
            "users", 
            total_users, 
            total_users, 
            "completed", 
            &format!("Completed: {} created, {} updated, {} linked, {} errors", 
                stats.new_users_created, stats.existing_users_updated, stats.identities_linked, stats.errors.len())
        );

        SyncProgress {
            entity: "users".to_string(),
            processed,
            total: total_users,
            status: if stats.errors.is_empty() { "completed".to_string() } else { "completed_with_errors".to_string() },
            errors: stats.errors,
        }
    } else {
        // This should never be reached since we return early on cancellation,
        // but just in case, return the cancelled status
        SyncProgress {
            entity: "users".to_string(),
            processed,
            total: total_users,
            status: "cancelled".to_string(),
            errors: stats.errors,
        }
    }
}

/// Fetch users from Microsoft Graph API (optimized version)
async fn fetch_microsoft_graph_users_optimized(provider_id: i32) -> Result<(Vec<MicrosoftGraphUser>, String), String> {
    // Get required configuration values from environment variables
    let client_id = config_utils::get_microsoft_client_id()
        .map_err(|e| format!("Microsoft Client ID configuration error: {}", e))?;
    
    let tenant_id = config_utils::get_microsoft_tenant_id()
        .map_err(|e| format!("Microsoft Tenant ID configuration error: {}", e))?;
    
    let client_secret = config_utils::get_microsoft_client_secret()
        .map_err(|e| format!("Microsoft Client Secret configuration error: {}", e))?;

    // Get an access token using client credentials flow
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "client_credentials"),
        ("scope", "https://graph.microsoft.com/.default"),
    ];

    // Create optimized HTTP client
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
    let token_response = client
        .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to request access token: {}", e))?;

    let token_data: serde_json::Value = token_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| {
            let error_desc = token_data.get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            format!("Failed to obtain access token: {}", error_desc)
        })?;

    // Build the Microsoft Graph API request for users
    // Select the fields we need for our MicrosoftGraphUser struct
    // Important: Include proxyAddresses and otherMails for email aliases
    let select_fields = "id,displayName,givenName,surname,mail,userPrincipalName,jobTitle,department,officeLocation,mobilePhone,businessPhones,proxyAddresses,otherMails";
    
    // Start with the first page
    let mut url = format!(
        "https://graph.microsoft.com/v1.0/users?$select={}",
        urlencoding::encode(select_fields)
    );

    println!("Microsoft Graph API query with email alias fields: {}", url);

    let mut all_users = Vec::new();
    let mut page_count = 0;

    loop {
        page_count += 1;
        println!("Fetching page {} from Microsoft Graph: {}", page_count, url);

        // Make the request to Microsoft Graph
        let graph_response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to send Microsoft Graph request (page {}): {}", page_count, e))?;

        let status = graph_response.status();
        let response_data: serde_json::Value = graph_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Microsoft Graph response (page {}): {}", page_count, e))?;

        if !status.is_success() {
            let error_msg = response_data
                .get("error")
                .and_then(|err| err.get("message"))
                .and_then(|msg| msg.as_str())
                .unwrap_or("Unknown Microsoft Graph error");
            return Err(format!("Microsoft Graph API error (page {}, {}): {}", page_count, status, error_msg));
        }

        // Parse the users from this page
        let users_array = response_data
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("Microsoft Graph response missing 'value' array (page {})", page_count))?;

        let mut page_users = Vec::new();
        for user_value in users_array {
            match serde_json::from_value::<MicrosoftGraphUser>(user_value.clone()) {
                Ok(user) => {
                    page_users.push(user);
                },
                Err(e) => {
                    println!("Warning: Failed to parse user from Microsoft Graph (page {}): {}, data: {}", page_count, e, user_value);
                    // Continue processing other users even if one fails to parse
                }
            }
        }

        println!("Page {}: Parsed {} users from Microsoft Graph", page_count, page_users.len());
        all_users.extend(page_users);

        // Check if there's a next page
        if let Some(next_link) = response_data.get("@odata.nextLink").and_then(|link| link.as_str()) {
            url = next_link.to_string();
            println!("Found next page link, continuing to page {}...", page_count + 1);
        } else {
            println!("No more pages found, finished pagination");
            break;
        }

        // Add a small delay between requests to be respectful to the API
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("Successfully fetched {} users from Microsoft Graph across {} pages", all_users.len(), page_count);
    
    // Log a sample of users for verification
    for (i, user) in all_users.iter().take(5).enumerate() {
        println!("Sample user {}: {} ({})", 
            i + 1, 
            user.display_name.as_deref().unwrap_or("N/A"), 
            user.user_principal_name
        );
    }
    
    if all_users.len() > 5 {
        println!("... and {} more users", all_users.len() - 5);
    }

    Ok((all_users, access_token.to_string()))
}

/// Process a single Microsoft Graph user
async fn process_microsoft_user(
    conn: &mut DbConnection,
    provider_id: i32,
    ms_user: &MicrosoftGraphUser,
    stats: &mut UserSyncStats,
    access_token: &str,
) -> Result<(), String> {
    // Step 1: Check if this Microsoft user already has an identity in our system
    if let Ok(existing_identity) = find_identity_by_provider_user_id(conn, provider_id, &ms_user.id) {
        // User already has Microsoft identity - update existing user and identity
        return update_existing_microsoft_user(conn, ms_user, existing_identity, stats, access_token).await;
    }

    // Step 2: Check if a local user exists with matching email
    let email = ms_user.mail.as_ref().unwrap_or(&ms_user.user_principal_name);
    
    if let Ok(existing_user) = user_repo::get_user_by_email(email, conn) {
        // Local user exists but no Microsoft identity - link them
        return link_existing_user_to_microsoft(conn, provider_id, ms_user, existing_user, stats, access_token).await;
    }

    // Step 3: No existing user found - create new user with Microsoft identity
    create_new_user_from_microsoft(conn, provider_id, ms_user, stats, access_token).await
}

/// Process a single Microsoft Graph user with optimized HTTP client (v2)
async fn process_microsoft_user_optimized_v2(
    conn: &mut DbConnection,
    provider_id: i32,
    ms_user: &MicrosoftGraphUser,
    stats: &mut UserSyncStats,
    access_token: &str,
    client: &reqwest::Client,
) -> Result<(), String> {
    // Step 1: Check if this Microsoft user already has an identity in our system
    if let Ok(existing_identity) = find_identity_by_provider_user_id(conn, provider_id, &ms_user.id) {
        // User already has Microsoft identity - update existing user and identity
        return update_existing_microsoft_user_optimized(conn, ms_user, existing_identity, stats, access_token, client).await;
    }

    // Step 2: Extract all email addresses from Microsoft Graph user
    let emails = extract_user_emails(ms_user);
    let email_addresses: Vec<String> = emails.iter().map(|(email, _, _)| email.clone()).collect();
    
    // Step 3: Check if any user exists with any of these email addresses
    if let Ok(Some(existing_user)) = user_emails_repo::find_user_by_any_of_emails(conn, &email_addresses) {
        // Local user exists but no Microsoft identity - link them
        return link_existing_user_to_microsoft_optimized(conn, provider_id, ms_user, existing_user, stats, access_token, client).await;
    }

    // Step 4: No existing user found - create new user with Microsoft identity
    create_new_user_from_microsoft_optimized(conn, provider_id, ms_user, stats, access_token, client).await
}

/// Find identity by provider and user ID
fn find_identity_by_provider_user_id(
    conn: &mut DbConnection,
    provider_id: i32,
    provider_user_id: &str,
) -> Result<UserAuthIdentity, diesel::result::Error> {
    use crate::schema::user_auth_identities;
    
    user_auth_identities::table
        .filter(user_auth_identities::auth_provider_id.eq(provider_id))
        .filter(user_auth_identities::provider_user_id.eq(provider_user_id))
        .first::<UserAuthIdentity>(conn)
}

/// Update existing user who already has Microsoft identity
async fn update_existing_microsoft_user(
    conn: &mut DbConnection,
    ms_user: &MicrosoftGraphUser,
    existing_identity: UserAuthIdentity,
    stats: &mut UserSyncStats,
    access_token: &str,
) -> Result<(), String> {
    println!("Updating existing Microsoft user: {}", ms_user.user_principal_name);

    // Get the associated user
    let user = user_repo::get_user_by_id(existing_identity.user_id, conn)
        .map_err(|e| format!("Failed to get user by ID {}: {}", existing_identity.user_id, e))?;

    // Update user information with latest from Microsoft Graph
    let updated_name = ms_user.display_name.as_ref().unwrap_or(&user.name);
    let updated_email = ms_user.mail.as_ref().unwrap_or(&ms_user.user_principal_name);

    // Only update core fields, preserve role/pronouns/avatars, but update timestamp
    let user_update = crate::models::UserUpdate {
        name: if updated_name != &user.name { Some(updated_name.clone()) } else { None },
        email: if updated_email != &user.email { Some(updated_email.clone()) } else { None },
        role: None, // Don't change role during sync
        pronouns: None, // Preserve pronouns
        avatar_url: None, // Preserve avatar
        banner_url: None, // Preserve banner
        avatar_thumb: None, // Preserve avatar thumb
        microsoft_uuid: None, // Don't update Microsoft UUID in this function
        updated_at: Some(chrono::Utc::now().naive_utc()), // Update timestamp to preserve created_at
    };

    // Update user if there are changes
    if user_update.name.is_some() || user_update.email.is_some() {
        user_repo::update_user(user.id, user_update, conn)
            .map_err(|e| format!("Failed to update user: {}", e))?;
        println!("Updated user information for: {}", user.name);
    }

    // Update identity data with latest from Microsoft Graph
    let identity_data = serde_json::to_value(ms_user)
        .map_err(|e| format!("Failed to serialize Microsoft user data: {}", e))?;

    update_identity_data(conn, existing_identity.id, Some(identity_data))
        .map_err(|e| format!("Failed to update identity data: {}", e))?;

    // Sync profile photo
    let client = reqwest::Client::new();
    if let Ok(photo_urls) = sync_user_profile_photo(&client, access_token, ms_user, &user.uuid).await {
        println!("sync_user_profile_photo returned URLs for user: {} - avatar: {:?}, thumb: {:?}", user.name, photo_urls.avatar_url, photo_urls.avatar_thumb);
        if let Err(e) = update_user_avatar_by_id(conn, user.id, photo_urls.avatar_url, photo_urls.avatar_thumb).await {
            println!("Warning: Failed to update avatar for user {}: {}", user.name, e);
        } else {
            println!("Successfully updated avatar for user: {}", user.name);
        }
    } else {
        println!("No profile photo available for user: {}", user.name);
    }

    stats.existing_users_updated += 1;
    Ok(())
}

/// Update identity data for an existing identity
fn update_identity_data(
    conn: &mut DbConnection,
    identity_id: i32,
    identity_data: Option<serde_json::Value>,
) -> Result<UserAuthIdentity, diesel::result::Error> {
    use crate::schema::user_auth_identities;
    
    diesel::update(user_auth_identities::table.find(identity_id))
        .set(user_auth_identities::identity_data.eq(identity_data))
        .get_result::<UserAuthIdentity>(conn)
}

/// Link existing local user to Microsoft identity
async fn link_existing_user_to_microsoft(
    conn: &mut DbConnection,
    provider_id: i32,
    ms_user: &MicrosoftGraphUser,
    existing_user: User,
    stats: &mut UserSyncStats,
    access_token: &str,
) -> Result<(), String> {
    println!("Linking existing user to Microsoft: {} -> {}", existing_user.email, ms_user.user_principal_name);

    // Create Microsoft identity for existing user
    let identity_data = serde_json::to_value(ms_user)
        .map_err(|e| format!("Failed to serialize Microsoft user data: {}", e))?;

    let new_identity = NewUserAuthIdentity {
        user_id: existing_user.id,
        auth_provider_id: provider_id,
        provider_user_id: ms_user.id.clone(),
        email: ms_user.mail.clone(),
        identity_data: Some(identity_data),
        password_hash: None, // Microsoft identities don't have password hashes
    };

    identity_repo::create_identity(new_identity, conn)
        .map_err(|e| format!("Failed to create Microsoft identity: {}", e))?;

    // Optionally update user information with Microsoft data
    let updated_name = ms_user.display_name.as_ref().unwrap_or(&existing_user.name);
    if updated_name != &existing_user.name {
        let user_update = crate::models::UserUpdate {
            name: Some(updated_name.clone()),
            email: None, // Don't change email when linking
            role: None,
            pronouns: None,
            avatar_url: None,
            banner_url: None,
            avatar_thumb: None,
            microsoft_uuid: None, // Don't update Microsoft UUID in this function
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        user_repo::update_user(existing_user.id, user_update, conn)
            .map_err(|e| format!("Failed to update user name: {}", e))?;
    }

    // Sync profile photo
    let client = reqwest::Client::new();
    if let Ok(photo_urls) = sync_user_profile_photo(&client, access_token, ms_user, &existing_user.uuid).await {
        println!("sync_user_profile_photo returned URLs for user: {} - avatar: {:?}, thumb: {:?}", existing_user.name, photo_urls.avatar_url, photo_urls.avatar_thumb);
        if let Err(e) = update_user_avatar_by_id(conn, existing_user.id, photo_urls.avatar_url, photo_urls.avatar_thumb).await {
            println!("Warning: Failed to update avatar for user {}: {}", existing_user.name, e);
        } else {
            println!("Successfully updated avatar for user: {}", existing_user.name);
        }
    } else {
        println!("No profile photo available for user: {}", existing_user.name);
    }

    stats.identities_linked += 1;
    Ok(())
}

/// Create new user from Microsoft Graph data
async fn create_new_user_from_microsoft(
    conn: &mut DbConnection,
    provider_id: i32,
    ms_user: &MicrosoftGraphUser,
    stats: &mut UserSyncStats,
    access_token: &str,
) -> Result<(), String> {
    println!("Creating new user from Microsoft: {}", ms_user.user_principal_name);

    // Generate UUID for new user
    let user_uuid = Uuid::new_v4().to_string();
    
    // Determine name (prefer displayName, fallback to givenName + surname, fallback to userPrincipalName)
    let name = ms_user.display_name.clone()
        .or_else(|| {
            match (&ms_user.given_name, &ms_user.surname) {
                (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                (Some(first), None) => Some(first.clone()),
                (None, Some(last)) => Some(last.clone()),
                _ => None,
            }
        })
        .unwrap_or_else(|| ms_user.user_principal_name.clone());

    // Use mail if available, otherwise use userPrincipalName
    let email = ms_user.mail.as_ref().unwrap_or(&ms_user.user_principal_name);

    // Create new user with default role 'user'
    let new_user = NewUser {
        uuid: user_uuid.clone(),
        name: name.clone(),
        email: email.clone(),
        role: "user".to_string(), // Default role for Microsoft Graph users
        pronouns: None,
        avatar_url: None,
        banner_url: None,
        avatar_thumb: None,
        microsoft_uuid: None, // Don't set Microsoft UUID in this function
    };

    let created_user = user_repo::create_user(new_user, conn)
        .map_err(|e| format!("Failed to create user: {}", e))?;

    // Create Microsoft identity for the new user
    let identity_data = serde_json::to_value(ms_user)
        .map_err(|e| format!("Failed to serialize Microsoft user data: {}", e))?;

    let new_identity = NewUserAuthIdentity {
        user_id: created_user.id,
        auth_provider_id: provider_id,
        provider_user_id: ms_user.id.clone(),
        email: ms_user.mail.clone(),
        identity_data: Some(identity_data),
        password_hash: None,
    };

    identity_repo::create_identity(new_identity, conn)
        .map_err(|e| format!("Failed to create Microsoft identity: {}", e))?;

    // Sync profile photo
    let client = reqwest::Client::new();
    if let Ok(photo_urls) = sync_user_profile_photo(&client, access_token, ms_user, &user_uuid).await {
        println!("sync_user_profile_photo returned URLs for new user: {} - avatar: {:?}, thumb: {:?}", name, photo_urls.avatar_url, photo_urls.avatar_thumb);
        if let Err(e) = update_user_avatar_by_id(conn, created_user.id, photo_urls.avatar_url, photo_urls.avatar_thumb).await {
            println!("Warning: Failed to update avatar for user {}: {}", name, e);
        } else {
            println!("Successfully updated avatar for new user: {}", name);
        }
    } else {
        println!("No profile photo available for user: {}", name);
    }

    println!("Created new user: {} ({})", name, email);
    stats.new_users_created += 1;
    Ok(())
}

/// Update existing user who already has Microsoft identity (optimized version)
async fn update_existing_microsoft_user_optimized(
    conn: &mut DbConnection,
    ms_user: &MicrosoftGraphUser,
    existing_identity: UserAuthIdentity,
    stats: &mut UserSyncStats,
    access_token: &str,
    client: &reqwest::Client,
) -> Result<(), String> {
    println!("Updating existing Microsoft user: {}", ms_user.user_principal_name);

    // Get the associated user
    let user = user_repo::get_user_by_id(existing_identity.user_id, conn)
        .map_err(|e| format!("Failed to get user by ID {}: {}", existing_identity.user_id, e))?;

    // Extract all email addresses from Microsoft Graph
    let emails = extract_user_emails(ms_user);
    let primary_email = emails.first().map(|(email, _, _)| email.clone())
        .unwrap_or_else(|| ms_user.user_principal_name.clone());

    // Update user information with latest from Microsoft Graph
    let updated_name = ms_user.display_name.as_ref().unwrap_or(&user.name);

    // Only update core fields, preserve role/pronouns/avatars, but update timestamp and Microsoft UUID
    let user_update = crate::models::UserUpdate {
        name: if updated_name != &user.name { Some(updated_name.clone()) } else { None },
        email: if primary_email != user.email { Some(primary_email.clone()) } else { None },
        role: None, // Don't change role during sync
        pronouns: None, // Preserve pronouns
        avatar_url: None, // Preserve avatar
        banner_url: None, // Preserve banner
        avatar_thumb: None, // Preserve avatar thumb
        microsoft_uuid: Some(ms_user.id.clone()), // Always update Microsoft UUID
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    // Update user if there are changes
    if user_update.name.is_some() || user_update.email.is_some() || user_update.microsoft_uuid.is_some() {
        user_repo::update_user(user.id, user_update, conn)
            .map_err(|e| format!("Failed to update user: {}", e))?;
        println!("Updated user information for: {}", user.name);
    }

    // Store all email addresses
    let email_data: Vec<(String, String, bool, String)> = emails
        .into_iter()
        .map(|(email, email_type, verified)| (email, email_type, verified, "microsoft".to_string()))
        .collect();

    if !email_data.is_empty() {
        println!("Storing {} email addresses for user: {}", email_data.len(), user.name);
        for (i, (email, email_type, verified, source)) in email_data.iter().enumerate() {
            println!("  Email {}: {} (type: {}, verified: {}, source: {})", i + 1, email, email_type, verified, source);
        }
        
        match user_emails_repo::add_multiple_emails(conn, user.id, email_data.clone()) {
            Ok(stored_emails) => {
                let added_count = stored_emails.len();
                println!("Successfully stored {} email addresses for user: {}", added_count, user.name);
                
                // Clean up any Microsoft emails that are no longer present
                let current_emails: Vec<String> = email_data.iter().map(|(email, _, _, _)| email.clone()).collect();
                match user_emails_repo::cleanup_obsolete_emails(conn, user.id, &current_emails, "microsoft") {
                    Ok(cleaned_count) => {
                        if cleaned_count > 0 {
                            println!("Cleaned up {} obsolete Microsoft email addresses for user: {}", cleaned_count, user.name);
                        }
                    },
                    Err(e) => {
                        println!("Warning: Failed to cleanup obsolete emails for user {}: {}", user.name, e);
                    }
                }
            },
            Err(e) => {
                println!("Error: Failed to store email addresses for user {}: {}", user.name, e);
                // Don't fail the entire user sync, but make sure the error is visible
                stats.errors.push(format!("Failed to store emails for user {}: {}", user.name, e));
            }
        }
    } else {
        println!("No email addresses to store for user: {}", user.name);
    }

    // Update identity data with latest from Microsoft Graph
    let identity_data = serde_json::to_value(ms_user)
        .map_err(|e| format!("Failed to serialize Microsoft user data: {}", e))?;

    update_identity_data(conn, existing_identity.id, Some(identity_data))
        .map_err(|e| format!("Failed to update identity data: {}", e))?;

    // Sync profile photo using optimized client
    if let Ok(photo_urls) = sync_user_profile_photo(client, access_token, ms_user, &user.uuid).await {
        if let Err(e) = update_user_avatar_by_id(conn, user.id, photo_urls.avatar_url, photo_urls.avatar_thumb).await {
            println!("Warning: Failed to update avatar for user {}: {}", user.name, e);
        }
    }

    stats.existing_users_updated += 1;
    Ok(())
}

/// Link existing local user to Microsoft identity (optimized version)
async fn link_existing_user_to_microsoft_optimized(
    conn: &mut DbConnection,
    provider_id: i32,
    ms_user: &MicrosoftGraphUser,
    existing_user: User,
    stats: &mut UserSyncStats,
    access_token: &str,
    client: &reqwest::Client,
) -> Result<(), String> {
    println!("Linking existing user to Microsoft: {} -> {}", existing_user.email, ms_user.user_principal_name);

    // Create Microsoft identity for existing user
    let identity_data = serde_json::to_value(ms_user)
        .map_err(|e| format!("Failed to serialize Microsoft user data: {}", e))?;

    let new_identity = NewUserAuthIdentity {
        user_id: existing_user.id,
        auth_provider_id: provider_id,
        provider_user_id: ms_user.id.clone(),
        email: ms_user.mail.clone(),
        identity_data: Some(identity_data),
        password_hash: None,
    };

    identity_repo::create_identity(new_identity, conn)
        .map_err(|e| format!("Failed to create Microsoft identity: {}", e))?;

    // Extract all email addresses from Microsoft Graph
    let emails = extract_user_emails(ms_user);
    let primary_email = emails.first().map(|(email, _, _)| email.clone())
        .unwrap_or_else(|| ms_user.user_principal_name.clone());

    // Update user information with Microsoft data and store Microsoft UUID
    let updated_name = ms_user.display_name.as_ref().unwrap_or(&existing_user.name);
    let user_update = crate::models::UserUpdate {
        name: if updated_name != &existing_user.name { Some(updated_name.clone()) } else { None },
        email: if primary_email != existing_user.email { Some(primary_email.clone()) } else { None },
        role: None,
        pronouns: None,
        avatar_url: None,
        banner_url: None,
        avatar_thumb: None,
        microsoft_uuid: Some(ms_user.id.clone()), // Store Microsoft UUID
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    // Always update to store the Microsoft UUID
    user_repo::update_user(existing_user.id, user_update, conn)
        .map_err(|e| format!("Failed to update user with Microsoft UUID: {}", e))?;

    // Store all email addresses
    let email_data: Vec<(String, String, bool, String)> = emails
        .into_iter()
        .map(|(email, email_type, verified)| (email, email_type, verified, "microsoft".to_string()))
        .collect();

    if !email_data.is_empty() {
        println!("Storing {} email addresses for linked user: {}", email_data.len(), existing_user.name);
        for (i, (email, email_type, verified, source)) in email_data.iter().enumerate() {
            println!("  Email {}: {} (type: {}, verified: {}, source: {})", i + 1, email, email_type, verified, source);
        }
        
        match user_emails_repo::add_multiple_emails(conn, existing_user.id, email_data.clone()) {
            Ok(stored_emails) => {
                let added_count = stored_emails.len();
                println!("Successfully stored {} email addresses for linked user: {}", added_count, existing_user.name);
            },
            Err(e) => {
                println!("Error: Failed to store email addresses for linked user {}: {}", existing_user.name, e);
                // Don't fail the entire user sync, but make sure the error is visible
                stats.errors.push(format!("Failed to store emails for linked user {}: {}", existing_user.name, e));
            }
        }
    } else {
        println!("No email addresses to store for linked user: {}", existing_user.name);
    }

    // Sync profile photo using optimized client
    if let Ok(photo_urls) = sync_user_profile_photo(client, access_token, ms_user, &existing_user.uuid).await {
        if let Err(e) = update_user_avatar_by_id(conn, existing_user.id, photo_urls.avatar_url, photo_urls.avatar_thumb).await {
            println!("Warning: Failed to update avatar for user {}: {}", existing_user.name, e);
        }
    }

    stats.identities_linked += 1;
    Ok(())
}

/// Create new user from Microsoft Graph data (optimized version)
async fn create_new_user_from_microsoft_optimized(
    conn: &mut DbConnection,
    provider_id: i32,
    ms_user: &MicrosoftGraphUser,
    stats: &mut UserSyncStats,
    access_token: &str,
    client: &reqwest::Client,
) -> Result<(), String> {
    println!("Creating new user from Microsoft: {}", ms_user.user_principal_name);

    // Generate UUID for new user (this is our local UUID, different from Microsoft's)
    let user_uuid = Uuid::new_v4().to_string();
    
    // Extract all email addresses from Microsoft Graph
    let emails = extract_user_emails(ms_user);
    let primary_email = emails.first().map(|(email, _, _)| email.clone())
        .unwrap_or_else(|| ms_user.user_principal_name.clone());
    
    // Determine name (prefer displayName, fallback to givenName + surname, fallback to userPrincipalName)
    let name = ms_user.display_name.clone()
        .or_else(|| {
            match (&ms_user.given_name, &ms_user.surname) {
                (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                (Some(first), None) => Some(first.clone()),
                (None, Some(last)) => Some(last.clone()),
                _ => None,
            }
        })
        .unwrap_or_else(|| ms_user.user_principal_name.clone());

    // Create new user with default role 'user' and store Microsoft UUID
    let new_user = NewUser {
        uuid: user_uuid.clone(),
        name: name.clone(),
        email: primary_email.clone(),
        role: "user".to_string(),
        pronouns: None,
        avatar_url: None,
        banner_url: None,
        avatar_thumb: None,
        microsoft_uuid: Some(ms_user.id.clone()), // Store Microsoft UUID
    };

    let created_user = user_repo::create_user(new_user, conn)
        .map_err(|e| format!("Failed to create user: {}", e))?;

    // Store all email addresses
    let email_data: Vec<(String, String, bool, String)> = emails
        .into_iter()
        .map(|(email, email_type, verified)| (email, email_type, verified, "microsoft".to_string()))
        .collect();

    if !email_data.is_empty() {
        println!("Storing {} email addresses for new user: {}", email_data.len(), name);
        for (i, (email, email_type, verified, source)) in email_data.iter().enumerate() {
            println!("  Email {}: {} (type: {}, verified: {}, source: {})", i + 1, email, email_type, verified, source);
        }
        
        match user_emails_repo::add_multiple_emails(conn, created_user.id, email_data.clone()) {
            Ok(stored_emails) => {
                let added_count = stored_emails.len();
                println!("Successfully stored {} email addresses for new user: {}", added_count, name);
            },
            Err(e) => {
                println!("Error: Failed to store email addresses for new user {}: {}", name, e);
                // Don't fail the entire user sync, but make sure the error is visible
                stats.errors.push(format!("Failed to store emails for new user {}: {}", name, e));
            }
        }
    } else {
        println!("No email addresses to store for new user: {}", name);
    }

    // Create Microsoft identity for the new user
    let identity_data = serde_json::to_value(ms_user)
        .map_err(|e| format!("Failed to serialize Microsoft user data: {}", e))?;

    let new_identity = NewUserAuthIdentity {
        user_id: created_user.id,
        auth_provider_id: provider_id,
        provider_user_id: ms_user.id.clone(),
        email: ms_user.mail.clone(),
        identity_data: Some(identity_data),
        password_hash: None,
    };

    identity_repo::create_identity(new_identity, conn)
        .map_err(|e| format!("Failed to create Microsoft identity: {}", e))?;

    // Sync profile photo using optimized client
    if let Ok(photo_urls) = sync_user_profile_photo(client, access_token, ms_user, &user_uuid).await {
        if let Err(e) = update_user_avatar_by_id(conn, created_user.id, photo_urls.avatar_url, photo_urls.avatar_thumb).await {
            println!("Warning: Failed to update avatar for user {}: {}", name, e);
        }
    }

    println!("Created new user: {} ({}) with Microsoft UUID: {} and {} email addresses", 
        name, primary_email, ms_user.id, email_data.len());
    stats.new_users_created += 1;
    Ok(())
}

/// Sync devices from Microsoft Graph (Intune managed devices)
async fn sync_devices(
    conn: &mut DbConnection,
    provider_id: i32,
    session_id: &str,
) -> SyncProgress {
    let mut stats = DeviceSyncStats {
        new_devices_created: 0,
        existing_devices_updated: 0,
        devices_assigned: 0,
        errors: Vec::new(),
    };

    update_sync_progress_with_type(session_id, "devices", 0, 0, "running", "Fetching devices from Microsoft Graph", "devices");

    // Step 1: Fetch devices from Microsoft Graph
    let (microsoft_devices, _access_token) = match fetch_microsoft_graph_devices(provider_id).await {
        Ok(result) => result,
        Err(error) => {
            update_sync_progress_with_type(session_id, "devices", 0, 0, "error", &format!("Failed to fetch devices: {}", error), "devices");
            return SyncProgress {
                entity: "devices".to_string(),
                processed: 0,
                total: 0,
                status: "error".to_string(),
                errors: vec![format!("Failed to fetch Microsoft Graph devices: {}", error)],
            };
        }
    };

    let total_devices = microsoft_devices.len();
    println!("Fetched {} devices from Microsoft Graph", total_devices);

    if total_devices == 0 {
        update_sync_progress_with_type(session_id, "devices", 0, 0, "completed", "No devices found to sync", "devices");
        return SyncProgress {
        entity: "devices".to_string(),
        processed: 0,
        total: 0,
        status: "completed".to_string(),
            errors: Vec::new(),
        };
    }

    update_sync_progress_with_type(session_id, "devices", 0, total_devices, "running", &format!("Processing {} devices", total_devices), "devices");

    // Step 2: Process devices
    let mut processed_count = 0;

    for ms_device in microsoft_devices {
        // Check for cancellation before processing each device
        if is_sync_cancelled(session_id) {
            let processed = stats.new_devices_created + stats.existing_devices_updated;
            let cancel_message = format!("Sync was cancelled by user request. Processed {} of {} devices ({} created, {} updated)", 
                processed_count, total_devices, stats.new_devices_created, stats.existing_devices_updated);
            
            update_sync_progress_with_type(
                session_id, 
                "devices", 
                processed_count, 
                total_devices, 
                "cancelled", 
                &cancel_message, 
                "devices"
            );
            
            return SyncProgress {
                entity: "devices".to_string(),
                processed,
                total: total_devices,
                status: "cancelled".to_string(),
                errors: stats.errors,
            };
        }
        
        processed_count += 1;
        
        update_sync_progress_with_type(
            session_id, 
            "devices", 
            processed_count - 1, 
            total_devices, 
            "running", 
            &format!("Processing device: {}", ms_device.device_name.as_deref().unwrap_or(&ms_device.id)),
            "devices"
        );

        match process_microsoft_device(conn, provider_id, &ms_device, &mut stats).await {
            Ok(_) => {
                println!("Successfully processed device: {}", ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
            },
            Err(error) => {
                let error_msg = format!("Failed to process device {}: {}", ms_device.device_name.as_deref().unwrap_or(&ms_device.id), error);
                println!("{}", error_msg);
                stats.errors.push(error_msg);
            }
        }

        // Update progress more frequently
        if processed_count % 5 == 0 || processed_count == total_devices {
            let processed = stats.new_devices_created + stats.existing_devices_updated;
            update_sync_progress_with_type(
                session_id, 
                "devices", 
                processed_count, 
                total_devices, 
                "running", 
                &format!("Processed {}/{} devices ({} created, {} updated, {} assigned, {} errors)", 
                    processed_count, total_devices, stats.new_devices_created, stats.existing_devices_updated, 
                    stats.devices_assigned, stats.errors.len()),
                "devices"
            );
        }

        // Small delay between devices to be respectful to the database
        if processed_count < total_devices {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    let processed = stats.new_devices_created + stats.existing_devices_updated;

    update_sync_progress_with_type(
        session_id, 
        "devices", 
        total_devices, 
        total_devices, 
        "completed", 
        &format!("Completed: {} created, {} updated, {} assigned, {} errors", 
            stats.new_devices_created, stats.existing_devices_updated, stats.devices_assigned, stats.errors.len()),
        "devices"
    );

    SyncProgress {
        entity: "devices".to_string(),
        processed,
        total: total_devices,
        status: if stats.errors.is_empty() { "completed".to_string() } else { "completed_with_errors".to_string() },
        errors: stats.errors,
    }
}

/// Sync groups from Microsoft Graph
async fn sync_groups(
    _conn: &mut DbConnection,
    _provider_id: i32,
    session_id: &str,
) -> SyncProgress {
    update_sync_progress(session_id, "groups", 0, 0, "completed", "Group sync not yet implemented");
    
    // TODO: Implement actual group sync logic
    SyncProgress {
        entity: "groups".to_string(),
        processed: 0,
        total: 0,
        status: "completed".to_string(),
        errors: vec!["Group sync not yet implemented".to_string()],
    }
}









/// Result struct for photo sync containing both avatar sizes
#[derive(Debug)]
pub struct PhotoSyncUrls {
    pub avatar_url: Option<String>,      // 120x120 or fallback
    pub avatar_thumb: Option<String>,    // 48x48 thumbnail
}

/// Fetch and save user profile photo from Microsoft Graph (updated to download 120x120 and generate thumbnail)
async fn sync_user_profile_photo(
    client: &reqwest::Client,
    access_token: &str,
    user: &MicrosoftGraphUser,
    local_user_uuid: &str,
) -> Result<PhotoSyncUrls, String> {
    println!("Fetching 120x120 profile photo and generating thumbnail for user: {}", user.user_principal_name);

    let mut avatar_url = None;
    let mut avatar_thumb = None;
    
    // Download 120x120 for profile views (main avatar) and generate thumbnail from it
    match download_profile_photo_size(client, access_token, user, local_user_uuid, "120x120").await {
        Ok(Some(url)) => {
            println!("Successfully downloaded 120x120 photo for user: {}", user.user_principal_name);
            avatar_url = Some(url.clone());
            
            // Generate 48x48 WebP thumbnail from the 120x120 image
            match generate_thumbnail_from_avatar(&url, local_user_uuid).await {
                Ok(Some(thumb_url)) => {
                    println!("Successfully generated thumbnail for user: {}", user.user_principal_name);
                    avatar_thumb = Some(thumb_url);
                },
                Ok(None) => println!("Failed to generate thumbnail for user: {}", user.user_principal_name),
                Err(e) => println!("Error generating thumbnail for user {}: {}", user.user_principal_name, e),
            }
        },
        Ok(None) => println!("No 120x120 photo available for user: {}", user.user_principal_name),
        Err(e) => println!("Failed to download 120x120 photo for user {}: {}", user.user_principal_name, e),
    }
    
    // If no 120x120 photo was available, try the default size as fallback
    if avatar_url.is_none() {
        println!("No 120x120 photo available, trying default size for user: {}", user.user_principal_name);
        match sync_user_profile_photo_fallback(client, access_token, user, local_user_uuid).await {
            Ok(Some(url)) => {
                println!("Successfully downloaded default photo for user: {}", user.user_principal_name);
                avatar_url = Some(url.clone());
                
                // Generate thumbnail from the default size image
                match generate_thumbnail_from_avatar(&url, local_user_uuid).await {
                    Ok(Some(thumb_url)) => {
                        println!("Successfully generated thumbnail from default photo for user: {}", user.user_principal_name);
                        avatar_thumb = Some(thumb_url);
                    },
                    Ok(None) => println!("Failed to generate thumbnail from default photo for user: {}", user.user_principal_name),
                    Err(e) => println!("Error generating thumbnail from default photo for user {}: {}", user.user_principal_name, e),
                }
            },
            Ok(None) => println!("No default photo available for user: {}", user.user_principal_name),
            Err(e) => println!("Failed to download default photo for user {}: {}", user.user_principal_name, e),
        }
    }
    
    Ok(PhotoSyncUrls {
        avatar_url,
        avatar_thumb,
    })
}

/// Download a specific size of profile photo
async fn download_profile_photo_size(
    client: &reqwest::Client,
    access_token: &str,
    user: &MicrosoftGraphUser,
    local_user_uuid: &str,
    size: &str,
) -> Result<Option<String>, String> {
    println!("Fetching {} profile photo for user: {}", size, user.user_principal_name);

    let photo_url = format!("https://graph.microsoft.com/v1.0/users/{}/photos/{}/$value", user.id, size);
    
    let photo_response = client
        .get(&photo_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch {} profile photo: {}", size, e))?;

    let status = photo_response.status();
    println!("{} photo request status for user {}: {}", size, user.user_principal_name, status);

    if !status.is_success() {
        if status.as_u16() == 404 {
            println!("{} profile photo not found for user: {}", size, user.user_principal_name);
            return Ok(None);
        } else if status.as_u16() == 400 {
            println!("{} profile photo request returned 400 Bad Request for user: {}", size, user.user_principal_name);
            return Ok(None);
        } else if status.as_u16() == 403 {
            println!("Access denied to {} profile photo for user: {} - insufficient permissions", size, user.user_principal_name);
            return Ok(None);
        } else {
            return Err(format!("Failed to fetch {} profile photo, status: {}", size, status));
        }
    }

    // Get the photo data
    let photo_bytes = photo_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read {} photo data: {}", size, e))?;

    if photo_bytes.is_empty() {
        println!("Empty {} profile photo for user: {}", size, user.user_principal_name);
        return Ok(None);
    }

    println!("Successfully downloaded {} profile photo ({} bytes) for user: {}", size, photo_bytes.len(), user.user_principal_name);

    // Save the photo to the filesystem
    save_profile_photo_to_disk(&photo_bytes, local_user_uuid, size).await
}

/// Save profile photo bytes to disk and return the URL
async fn save_profile_photo_to_disk(
    photo_bytes: &[u8],
    local_user_uuid: &str,
    size_label: &str,
) -> Result<Option<String>, String> {
    // Create the avatar directory if it doesn't exist
    let avatar_dir = "uploads/users/avatars";
    fs::create_dir_all(avatar_dir)
        .await
        .map_err(|e| format!("Failed to create avatar directory: {}", e))?;

    // Clean up any existing files for this user and size before saving new one
    cleanup_old_user_images(avatar_dir, local_user_uuid, size_label).await?;

    // Generate predictable filename for the photo with size label
    let file_extension = "jpg"; // Microsoft Graph typically returns JPEG photos
    let filename = if size_label == "default" {
        format!("{}_{}.{}", local_user_uuid, size_label, file_extension)
    } else {
        format!("{}_{}.{}", local_user_uuid, size_label, file_extension)
    };
    let file_path = format!("{}/{}", avatar_dir, filename);

    println!("Saving profile photo to: {}", file_path);

    // Save the photo to the filesystem
    let mut file = fs::File::create(&file_path)
        .await
        .map_err(|e| format!("Failed to create photo file {}: {}", file_path, e))?;

    file.write_all(photo_bytes)
        .await
        .map_err(|e| format!("Failed to write photo data to {}: {}", file_path, e))?;

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush photo file {}: {}", file_path, e))?;

    println!("Successfully saved profile photo for user to: {}", file_path);

    // Return the URL path for the database
    let avatar_url = format!("/{}", file_path);
    println!("Generated avatar URL: {}", avatar_url);
    Ok(Some(avatar_url))
}

/// Clean up old user images for a specific size
async fn cleanup_old_user_images(
    avatar_dir: &str,
    user_uuid: &str,
    size_label: &str,
) -> Result<(), String> {
    use tokio::fs;
    
    // Read the directory
    let mut dir = match fs::read_dir(avatar_dir).await {
        Ok(dir) => dir,
        Err(_) => return Ok(()) // Directory doesn't exist, nothing to clean
    };

    // Look for files matching the pattern: {user_uuid}_{size_label}.{ext}
    let pattern_prefix = format!("{}_{}", user_uuid, size_label);
    
    while let Ok(Some(entry)) = dir.next_entry().await {
        if let Some(filename) = entry.file_name().to_str() {
            // Check if this file matches our pattern (user_uuid_size.ext)
            if filename.starts_with(&pattern_prefix) && filename.contains('.') {
                let file_path = entry.path();
                println!("Cleaning up old image file: {:?}", file_path);
                
                if let Err(e) = fs::remove_file(&file_path).await {
                    eprintln!("Warning: Failed to remove old image file {:?}: {}", file_path, e);
                    // Continue with cleanup even if one file fails
                }
            }
        }
    }
    
    Ok(())
}

/// Generate a 48x48 WebP thumbnail from an existing avatar image
async fn generate_thumbnail_from_avatar(
    avatar_url: &str,
    user_uuid: &str,
) -> Result<Option<String>, String> {
    // Convert URL path to file system path
    let file_path = if avatar_url.starts_with('/') {
        format!(".{}", avatar_url) // Remove leading slash and add current directory
    } else {
        avatar_url.to_string()
    };
    
    println!("Generating thumbnail from: {}", file_path);
    
    // Read the original image
    let img_bytes = match tokio::fs::read(&file_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("Failed to read avatar file {}: {}", file_path, e);
            return Ok(None);
        }
    };
    
    // Process image in a blocking task to avoid blocking the async runtime
    let user_uuid = user_uuid.to_string();
    let thumbnail_result = tokio::task::spawn_blocking(move || {
        // Load the image
        let img = match image::load_from_memory(&img_bytes) {
            Ok(img) => img,
            Err(e) => {
                println!("Failed to load image from memory: {}", e);
                return None;
            }
        };
        
        // Resize to 48x48 with high quality
        let thumbnail = img.resize_exact(48, 48, image::imageops::FilterType::Lanczos3);
        
        // Convert to WebP format
        let mut webp_bytes = Vec::new();
        match thumbnail.write_to(&mut std::io::Cursor::new(&mut webp_bytes), ImageFormat::WebP) {
            Ok(_) => Some(webp_bytes),
            Err(e) => {
                println!("Failed to encode image as WebP: {}", e);
                None
            }
        }
    }).await;
    
    let webp_bytes = match thumbnail_result {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(None),
        Err(e) => {
            println!("Thumbnail generation task failed: {}", e);
            return Ok(None);
        }
    };
    
    // Create thumbnail directory
    let thumb_dir = "uploads/users/thumbs";
    if let Err(e) = tokio::fs::create_dir_all(thumb_dir).await {
        return Err(format!("Failed to create thumbnail directory: {}", e));
    }
    
    // Clean up any existing thumbnails for this user
    cleanup_old_user_images(thumb_dir, &user_uuid, "thumb").await?;
    
    // Save the thumbnail
    let thumb_filename = format!("{}_thumb.webp", user_uuid);
    let thumb_path = format!("{}/{}", thumb_dir, thumb_filename);
    
    match tokio::fs::write(&thumb_path, &webp_bytes).await {
        Ok(_) => {
            println!("Successfully saved thumbnail to: {}", thumb_path);
            let thumb_url = format!("/{}", thumb_path);
            Ok(Some(thumb_url))
        },
        Err(e) => {
            println!("Failed to save thumbnail: {}", e);
            Ok(None)
        }
    }
}

/// Update user avatar URLs in the database
async fn update_user_avatar_by_id(
    conn: &mut DbConnection,
    user_id: i32,
    avatar_url: Option<String>,
    avatar_thumb: Option<String>,
) -> Result<(), String> {
    println!("update_user_avatar_by_id called for user_id: {}, avatar_url: {:?}, avatar_thumb: {:?}", user_id, avatar_url, avatar_thumb);
    
    if avatar_url.is_some() || avatar_thumb.is_some() {
        println!("Updating avatar URLs for user ID {} - main: {:?}, thumb: {:?}", user_id, avatar_url, avatar_thumb);
        
        let user_update = crate::models::UserUpdate {
            name: None,
            email: None,
            role: None,
            pronouns: None,
            avatar_url: avatar_url.clone(),
            banner_url: None,
            avatar_thumb: avatar_thumb.clone(),
            microsoft_uuid: None, // Don't update Microsoft UUID when updating avatar
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        match user_repo::update_user(user_id, user_update, conn) {
            Ok(updated_user) => {
                println!("Successfully updated avatar URLs for user ID {} - main: {:?}, thumb: {:?}", user_id, updated_user.avatar_url, updated_user.avatar_thumb);
            },
            Err(e) => {
                let error_msg = format!("Failed to update user avatar: {}", e);
                println!("Error: {}", error_msg);
                return Err(error_msg);
            }
        }
    } else {
        println!("No avatar URLs provided for user ID {}, skipping update", user_id);
    }

    Ok(())
}

/// Fallback function to get profile photo in default size if 120x120 is not available
async fn sync_user_profile_photo_fallback(
    client: &reqwest::Client,
    access_token: &str,
    user: &MicrosoftGraphUser,
    local_user_uuid: &str,
) -> Result<Option<String>, String> {
    println!("Fetching default size profile photo for user: {}", user.user_principal_name);

    // Try to get the user's profile photo in default size
    let photo_url = format!("https://graph.microsoft.com/v1.0/users/{}/photo/$value", user.id);
    
    let photo_response = client
        .get(&photo_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch default profile photo: {}", e))?;

    let status = photo_response.status();
    println!("Default photo request status for user {}: {}", user.user_principal_name, status);

    if !status.is_success() {
        // User likely doesn't have a profile photo
        if status.as_u16() == 404 {
            println!("No profile photo found for user: {}", user.user_principal_name);
            return Ok(None);
        } else if status.as_u16() == 400 {
            println!("Profile photo request returned 400 Bad Request for user: {} - user may not have a photo", user.user_principal_name);
            return Ok(None);
        } else if status.as_u16() == 403 {
            println!("Access denied to profile photo for user: {} - insufficient permissions", user.user_principal_name);
            return Ok(None);
        } else {
            println!("Failed to fetch profile photo for user: {}, status: {} - skipping", user.user_principal_name, status);
            return Ok(None);
        }
    }

    // Get the photo data
    let photo_bytes = photo_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read photo data: {}", e))?;

    if photo_bytes.is_empty() {
        println!("Empty profile photo for user: {}", user.user_principal_name);
        return Ok(None);
    }

    println!("Successfully downloaded default profile photo ({} bytes) for user: {}", photo_bytes.len(), user.user_principal_name);

    // Save the photo to the filesystem
    save_profile_photo_to_disk(&photo_bytes, local_user_uuid, "default").await
}

/// Fetch devices from Microsoft Graph API (Intune managed devices)
async fn fetch_microsoft_graph_devices(provider_id: i32) -> Result<(Vec<MicrosoftGraphDevice>, String), String> {
    // Get required configuration values from environment variables
    let client_id = config_utils::get_microsoft_client_id()
        .map_err(|e| format!("Microsoft Client ID configuration error: {}", e))?;
    
    let tenant_id = config_utils::get_microsoft_tenant_id()
        .map_err(|e| format!("Microsoft Tenant ID configuration error: {}", e))?;
    
    let client_secret = config_utils::get_microsoft_client_secret()
        .map_err(|e| format!("Microsoft Client Secret configuration error: {}", e))?;

    // Get an access token using client credentials flow
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "client_credentials"),
        ("scope", "https://graph.microsoft.com/.default"),
    ];

    // Create optimized HTTP client
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
    let token_response = client
        .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to request access token: {}", e))?;

    let token_data: serde_json::Value = token_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| {
            let error_desc = token_data.get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            format!("Failed to obtain access token: {}", error_desc)
        })?;

    // Build the Microsoft Graph API request for managed devices
    // Note: Using correct field names for managedDevice type from Microsoft Graph v1.0 API
    let select_fields = "id,deviceName,operatingSystem,osVersion,manufacturer,model,serialNumber,azureADDeviceId,userPrincipalName,userId,complianceState,lastSyncDateTime,enrolledDateTime,deviceEnrollmentType,managementAgent";
    
    let mut url = format!(
        "https://graph.microsoft.com/v1.0/deviceManagement/managedDevices?$select={}",
        urlencoding::encode(select_fields)
    );

    let mut all_devices = Vec::new();
    let mut page_count = 0;

    loop {
        page_count += 1;
        println!("Fetching Intune devices page {} from Microsoft Graph: {}", page_count, url);

        let graph_response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to send Microsoft Graph request (page {}): {}", page_count, e))?;

        let status = graph_response.status();
        let response_data: serde_json::Value = graph_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Microsoft Graph response (page {}): {}", page_count, e))?;

        if !status.is_success() {
            let error_msg = response_data
                .get("error")
                .and_then(|err| err.get("message"))
                .and_then(|msg| msg.as_str())
                .unwrap_or("Unknown Microsoft Graph error");
            return Err(format!("Microsoft Graph API error (page {}, {}): {}", page_count, status, error_msg));
        }

        let devices_array = response_data
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("Microsoft Graph response missing 'value' array (page {})", page_count))?;

        let mut page_devices = Vec::new();
        for device_value in devices_array {
            match serde_json::from_value::<MicrosoftGraphDevice>(device_value.clone()) {
                Ok(device) => {
                    page_devices.push(device);
                },
                Err(e) => {
                    println!("Warning: Failed to parse Intune device from Microsoft Graph (page {}): {}, data: {}", page_count, e, device_value);
                }
            }
        }

        println!("Intune devices page {}: Parsed {} devices", page_count, page_devices.len());
        all_devices.extend(page_devices);

        if let Some(next_link) = response_data.get("@odata.nextLink").and_then(|link| link.as_str()) {
            url = next_link.to_string();
        } else {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("Successfully fetched {} devices from Microsoft Graph across {} pages", all_devices.len(), page_count);
    
    // Log a sample of devices for verification
    for (i, device) in all_devices.iter().take(5).enumerate() {
        println!("Sample device {}: {} ({})", 
            i + 1, 
            device.device_name.as_deref().unwrap_or("N/A"), 
            device.id
        );
    }
    
    if all_devices.len() > 5 {
        println!("... and {} more devices", all_devices.len() - 5);
    }

    Ok((all_devices, access_token.to_string()))
}

/// Process a single Microsoft Graph device
async fn process_microsoft_device(
    conn: &mut DbConnection,
    provider_id: i32,
    ms_device: &MicrosoftGraphDevice,
    stats: &mut DeviceSyncStats,
) -> Result<(), String> {
    println!("Processing device: {}", ms_device.device_name.as_deref().unwrap_or(&ms_device.id));

    // Step 1: Check if this device already exists by Intune device ID
    let existing_device = device_repo::get_device_by_intune_id(conn, &ms_device.id).ok();

    // Step 2: If not found by Intune ID, try Entra device ID
    let existing_device = if existing_device.is_none() {
        if let Some(entra_id) = &ms_device.azure_ad_device_id {
            device_repo::get_device_by_entra_id(conn, entra_id).ok()
        } else {
            None
        }
    } else {
        existing_device
    };

    // Step 3: Try to find the primary user for this device
    let primary_user_uuid = {
        // First, try to match by Microsoft UUID if available (most reliable)
        if let Some(microsoft_user_id) = &ms_device.user_id {
            match user_repo::get_user_by_microsoft_uuid(conn, microsoft_user_id) {
                Ok(user) => {
                    println!("Found user {} for device {} by Microsoft UUID", user.name, ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
                    Some(user.uuid)
                },
                Err(_) => {
                    println!("User with Microsoft UUID {} not found for device {}", microsoft_user_id, ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
                    
                    // Fallback to email matching if Microsoft UUID doesn't match
                    if let Some(user_principal_name) = &ms_device.user_principal_name {
                        match user_emails_repo::find_user_by_any_email(conn, user_principal_name) {
                            Ok(user) => {
                                println!("Found user {} for device {} by email fallback", user.name, ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
                                Some(user.uuid)
                            },
                            Err(_) => {
                                println!("User {} not found for device {} (tried both Microsoft UUID and email)", user_principal_name, ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
            }
        }
        // If no Microsoft UUID available, try email matching only
        else if let Some(user_principal_name) = &ms_device.user_principal_name {
            match user_emails_repo::find_user_by_any_email(conn, user_principal_name) {
                Ok(user) => {
                    println!("Found user {} for device {} by email", user.name, ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
                    Some(user.uuid)
                },
                Err(_) => {
                    println!("User {} not found for device {}", user_principal_name, ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
                    None
                }
            }
        } else {
            println!("No user information available for device {}", ms_device.device_name.as_deref().unwrap_or(&ms_device.id));
            None
        }
    };

    // Step 4: Prepare device data
    let device_name = ms_device.device_name
        .as_ref()
        .cloned()
        .unwrap_or_else(|| format!("Device-{}", ms_device.id));

    let hostname = device_name.clone();

    let serial_number = ms_device.serial_number
        .as_ref()
        .cloned()
        .unwrap_or_else(|| format!("Unknown-{}", ms_device.id));

    let model = ms_device.model
        .as_ref()
        .cloned()
        .unwrap_or_else(|| "Unknown Model".to_string());

    let manufacturer = ms_device.manufacturer
        .as_ref()
        .cloned()
        .unwrap_or_else(|| "Unknown Manufacturer".to_string());

    // Set warranty status to Unknown - this field is reserved for actual warranty API data
    // Compliance state from Intune is not warranty information
    let warranty_status = "Unknown".to_string();

    if let Some(existing) = existing_device {
        // Update existing device
        let device_update = crate::models::DeviceUpdate {
            name: Some(device_name.clone()),
            hostname: Some(hostname),
            serial_number: Some(serial_number),
            model: Some(model),
            warranty_status: Some(warranty_status),
            manufacturer: Some(manufacturer),
            primary_user_uuid: primary_user_uuid.clone(),
            intune_device_id: Some(ms_device.id.clone()),
            entra_device_id: ms_device.azure_ad_device_id.clone(),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        device_repo::update_device(conn, existing.id, device_update)
            .map_err(|e| format!("Failed to update device: {}", e))?;

        println!("Updated existing device: {}", device_name);
        stats.existing_devices_updated += 1;

        if primary_user_uuid.is_some() {
            stats.devices_assigned += 1;
        }
    } else {
        // Create new device
        let new_device = crate::models::NewDevice {
            name: device_name.clone(),
            hostname,
            serial_number,
            model,
            warranty_status,
            manufacturer: Some(manufacturer),
            primary_user_uuid: primary_user_uuid.clone(),
            intune_device_id: Some(ms_device.id.clone()),
            entra_device_id: ms_device.azure_ad_device_id.clone(),
        };

        device_repo::create_device(conn, new_device)
            .map_err(|e| format!("Failed to create device: {}", e))?;

        println!("Created new device: {}", device_name);
        stats.new_devices_created += 1;

        if primary_user_uuid.is_some() {
            stats.devices_assigned += 1;
        }
    }

    Ok(())
}

/// Get Entra Object ID from Azure AD Device ID
pub async fn get_entra_object_id(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    path: web::Path<String>,
) -> impl Responder {
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Database connection failed"
        })),
    };

    // Validate token
    let _claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Get Microsoft provider
    let provider = match auth_provider_repo::get_default_provider_by_type("microsoft", &mut conn) {
        Ok(provider) => provider,
        Err(_) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Microsoft auth provider not found"
        })),
    };

    let azure_ad_device_id = path.into_inner();
    
    // Fetch the Object ID from Microsoft Graph
    match fetch_entra_object_id_from_graph(provider.id, &azure_ad_device_id).await {
        Ok(object_id) => HttpResponse::Ok().json(json!({
            "success": true,
            "azure_ad_device_id": azure_ad_device_id,
            "object_id": object_id,
            "entra_url": format!("https://entra.microsoft.com/#view/Microsoft_AAD_Devices/DeviceDetailsMenuBlade/~/Properties/objectId/{}", object_id)
        })),
        Err(error) => HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("Failed to fetch Object ID: {}", error)
        }))
    }
}

/// Fetch Entra Object ID from Microsoft Graph using Azure AD Device ID
async fn fetch_entra_object_id_from_graph(provider_id: i32, azure_ad_device_id: &str) -> Result<String, String> {
    // Get required configuration values from environment variables
    let client_id = config_utils::get_microsoft_client_id()
        .map_err(|e| format!("Microsoft Client ID configuration error: {}", e))?;
    
    let tenant_id = config_utils::get_microsoft_tenant_id()
        .map_err(|e| format!("Microsoft Tenant ID configuration error: {}", e))?;
    
    let client_secret = config_utils::get_microsoft_client_secret()
        .map_err(|e| format!("Microsoft Client Secret configuration error: {}", e))?;

    // Get an access token using client credentials flow
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "client_credentials"),
        ("scope", "https://graph.microsoft.com/.default"),
    ];

    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
    let token_response = client
        .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to request access token: {}", e))?;

    let token_data: serde_json::Value = token_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| {
            let error_desc = token_data.get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            format!("Failed to obtain access token: {}", error_desc)
        })?;

    // Query Microsoft Graph for the device using the Azure AD Device ID
    // We need to filter by deviceId (which is the Azure AD Device ID) to get the Object ID (id field)
    let url = format!(
        "https://graph.microsoft.com/v1.0/devices?$filter=deviceId eq '{}'&$select=id,deviceId",
        azure_ad_device_id
    );

    println!("Fetching Entra Object ID for Azure AD Device ID: {}", azure_ad_device_id);

    let graph_response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to send Microsoft Graph request: {}", e))?;

    let status = graph_response.status();
    let response_data: serde_json::Value = graph_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Microsoft Graph response: {}", e))?;

    if !status.is_success() {
        let error_msg = response_data
            .get("error")
            .and_then(|err| err.get("message"))
            .and_then(|msg| msg.as_str())
            .unwrap_or("Unknown Microsoft Graph error");
        return Err(format!("Microsoft Graph API error ({}): {}", status, error_msg));
    }

    // Parse the response to get the Object ID
    let devices_array = response_data
        .get("value")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Microsoft Graph response missing 'value' array".to_string())?;

    if devices_array.is_empty() {
        return Err(format!("No device found with Azure AD Device ID: {}", azure_ad_device_id));
    }

    // Get the first (and should be only) device
    let device = &devices_array[0];
    let object_id = device
        .get("id")
        .and_then(|id| id.as_str())
        .ok_or_else(|| "Device Object ID not found in response".to_string())?;

    println!("Successfully found Object ID {} for Azure AD Device ID {}", object_id, azure_ad_device_id);
    
    Ok(object_id.to_string())
}

/// Extract all email addresses from Microsoft Graph user data
fn extract_user_emails(ms_user: &MicrosoftGraphUser) -> Vec<(String, String, bool)> {
    let mut emails = Vec::new();
    
    println!("Extracting emails for user: {} (ID: {})", ms_user.user_principal_name, ms_user.id);
    
    // Primary email (mail field)
    if let Some(mail) = &ms_user.mail {
        if !mail.is_empty() && mail.contains('@') {
            emails.push((mail.clone(), "primary".to_string(), true));
            println!("  Added primary email: {}", mail);
        }
    }
    
    // User Principal Name (if different from mail)
    if !ms_user.user_principal_name.is_empty() && 
       ms_user.user_principal_name.contains('@') &&
       !emails.iter().any(|(e, _, _)| e == &ms_user.user_principal_name) {
        let email_type = if emails.is_empty() { "primary".to_string() } else { "upn".to_string() };
        emails.push((ms_user.user_principal_name.clone(), email_type.clone(), true));
        println!("  Added UPN email: {} (type: {})", ms_user.user_principal_name, email_type);
    }
    
    // Proxy addresses (SMTP addresses from Exchange)
    if let Some(proxy_addresses) = &ms_user.proxy_addresses {
        println!("  Processing {} proxy addresses: {:?}", proxy_addresses.len(), proxy_addresses);
        for proxy in proxy_addresses {
            if let Some(email) = extract_smtp_address(proxy) {
                if !emails.iter().any(|(e, _, _)| e == &email) {
                    let email_type = if proxy.starts_with("SMTP:") {
                        "primary".to_string() // SMTP: (uppercase) indicates primary
                    } else {
                        "alias".to_string() // smtp: (lowercase) indicates alias
                    };
                    emails.push((email.clone(), email_type.clone(), true));
                    println!("  Added proxy email: {} (type: {}, original: {})", email, email_type, proxy);
                } else {
                    println!("  Skipped duplicate proxy email: {} (original: {})", email, proxy);
                }
            } else {
                println!("  Failed to extract email from proxy address: {}", proxy);
            }
        }
    } else {
        println!("  No proxy addresses found");
    }
    
    // Other mail addresses
    if let Some(other_mails) = &ms_user.other_mails {
        println!("  Processing {} other mail addresses: {:?}", other_mails.len(), other_mails);
        for email in other_mails {
            if !email.is_empty() && 
               email.contains('@') && 
               !emails.iter().any(|(e, _, _)| e == email) {
                emails.push((email.clone(), "other".to_string(), true));
                println!("  Added other email: {}", email);
            } else {
                println!("  Skipped invalid or duplicate other email: {}", email);
            }
        }
    } else {
        println!("  No other mail addresses found");
    }
    
    // If no emails found, use the userPrincipalName as a fallback
    if emails.is_empty() && !ms_user.user_principal_name.is_empty() {
        emails.push((ms_user.user_principal_name.clone(), "primary".to_string(), true));
        println!("  Added fallback UPN email: {}", ms_user.user_principal_name);
    }
    
    println!("  Total emails extracted: {}", emails.len());
    for (i, (email, email_type, verified)) in emails.iter().enumerate() {
        println!("    {}: {} (type: {}, verified: {})", i + 1, email, email_type, verified);
    }
    
    emails
}

/// Extract email address from Exchange proxy address format
fn extract_smtp_address(proxy_address: &str) -> Option<String> {
    if proxy_address.starts_with("SMTP:") {
        Some(proxy_address[5..].to_string())
    } else if proxy_address.starts_with("smtp:") {
        Some(proxy_address[5..].to_string())
    } else if proxy_address.contains('@') {
        // Sometimes proxy addresses don't have the SMTP: prefix
        Some(proxy_address.to_string())
    } else {
        None
    }
} 