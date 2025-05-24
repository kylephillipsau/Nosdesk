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
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::db::{Pool, DbConnection};
use crate::handlers::auth::validate_token_internal;
use crate::repository::auth_providers as auth_provider_repo;
use crate::repository::users as user_repo;
use crate::repository::devices as device_repo;
use crate::repository::user_auth_identities as identity_repo;
use crate::config_utils;
use crate::models::{NewUser, NewUserAuthIdentity, User, UserAuthIdentity};

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
}

// User sync statistics
#[derive(Serialize, Debug)]
pub struct UserSyncStats {
    pub new_users_created: usize,
    pub existing_users_updated: usize,
    pub identities_linked: usize,
    pub errors: Vec<String>,
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

    // Perform the sync
    match perform_sync(&mut conn, provider.id, &request.entities).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "message": format!("Sync failed: {}", error)
        }))
    }
}

/// Sync profile photos for existing Microsoft users
async fn sync_existing_user_photos(
    conn: &mut DbConnection,
    provider_id: i32,
) -> Result<serde_json::Value, String> {
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

    // Get all users with Microsoft identities
    let microsoft_identities = identity_repo::get_identities_by_provider(provider_id, conn)
        .map_err(|e| format!("Failed to get Microsoft identities: {}", e))?;

    let mut photos_synced = 0;
    let mut photos_failed = 0;
    let mut errors = Vec::new();
    let total_users = microsoft_identities.len();

    println!("Found {} users with Microsoft identities", total_users);

    for identity in &microsoft_identities {
        // Get user details
        let user = match user_repo::get_user_by_id(identity.user_id, conn) {
            Ok(user) => user,
            Err(e) => {
                let error_msg = format!("Failed to get user for identity {}: {}", identity.id, e);
                println!("{}", error_msg);
                errors.push(error_msg);
                photos_failed += 1;
                continue;
            }
        };

        // Parse Microsoft Graph user data from identity_data
        let ms_user_data = match &identity.identity_data {
            Some(data) => data.clone(),
            None => {
                let error_msg = format!("No identity data for user {}", user.name);
                println!("{}", error_msg);
                errors.push(error_msg);
                photos_failed += 1;
                continue;
            }
        };

        let ms_user: MicrosoftGraphUser = match serde_json::from_value(ms_user_data) {
            Ok(user) => user,
            Err(e) => {
                let error_msg = format!("Failed to parse Microsoft user data for {}: {}", user.name, e);
                println!("{}", error_msg);
                errors.push(error_msg);
                photos_failed += 1;
                continue;
            }
        };

        // Sync profile photo
        match sync_user_profile_photo(&client, access_token, &ms_user, &user.uuid).await {
            Ok(avatar_url) => {
                match update_user_avatar_by_id(conn, user.id, avatar_url).await {
                    Ok(_) => {
                        println!("Successfully synced profile photo for user: {}", user.name);
                        photos_synced += 1;
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to update avatar for user {}: {}", user.name, e);
                        println!("{}", error_msg);
                        errors.push(error_msg);
                        photos_failed += 1;
                    }
                }
            },
            Err(e) => {
                let error_msg = format!("Failed to sync profile photo for user {}: {}", user.name, e);
                println!("{}", error_msg);
                errors.push(error_msg);
                photos_failed += 1;
            }
        }

        // Add a small delay between requests to be respectful to the API
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(json!({
        "success": true,
        "message": format!("Profile photo sync completed: {} synced, {} failed", photos_synced, photos_failed),
        "photos_synced": photos_synced,
        "photos_failed": photos_failed,
        "total_users": total_users,
        "errors": errors
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
) -> Result<SyncResult, String> {
    let mut results = Vec::new();
    let mut total_processed = 0;
    let mut total_errors = 0;

    for entity in entities {
        let sync_progress = match entity.as_str() {
            "users" => sync_users(conn, provider_id).await,
            "devices" => sync_devices(conn, provider_id).await,
            "groups" => sync_groups(conn, provider_id).await,
            _ => {
                total_errors += 1;
                SyncProgress {
                    entity: entity.clone(),
                    processed: 0,
                    total: 0,
                    status: "error".to_string(),
                    errors: vec![format!("Unsupported entity type: {}", entity)],
                }
            }
        };

        total_processed += sync_progress.processed;
        total_errors += sync_progress.errors.len();
        results.push(sync_progress);
    }

    Ok(SyncResult {
        success: total_errors == 0,
        message: if total_errors == 0 {
            format!("Successfully synchronized {} items", total_processed)
        } else {
            format!("Synchronized {} items with {} errors", total_processed, total_errors)
        },
        results,
        total_processed,
        total_errors,
    })
}

/// Sync users from Microsoft Graph
async fn sync_users(
    conn: &mut DbConnection,
    provider_id: i32,
) -> SyncProgress {
    let mut stats = UserSyncStats {
        new_users_created: 0,
        existing_users_updated: 0,
        identities_linked: 0,
        errors: Vec::new(),
    };

    // Step 1: Fetch users from Microsoft Graph
    let (microsoft_users, access_token) = match fetch_microsoft_graph_users(provider_id).await {
        Ok(result) => result,
        Err(error) => {
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

    // Step 2: Process each Microsoft Graph user
    for ms_user in microsoft_users {
        match process_microsoft_user(conn, provider_id, &ms_user, &mut stats, &access_token).await {
            Ok(_) => {
                println!("Successfully processed user: {}", ms_user.user_principal_name);
            },
            Err(error) => {
                let error_msg = format!("Failed to process user {}: {}", ms_user.user_principal_name, error);
                println!("{}", error_msg);
                stats.errors.push(error_msg);
            }
        }
    }

    let processed = stats.new_users_created + stats.existing_users_updated + stats.identities_linked;

    SyncProgress {
        entity: "users".to_string(),
        processed,
        total: total_users,
        status: if stats.errors.is_empty() { "completed".to_string() } else { "completed_with_errors".to_string() },
        errors: stats.errors,
    }
}

/// Fetch users from Microsoft Graph API
async fn fetch_microsoft_graph_users(provider_id: i32) -> Result<(Vec<MicrosoftGraphUser>, String), String> {
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
    let select_fields = "id,displayName,givenName,surname,mail,userPrincipalName,jobTitle,department,officeLocation,mobilePhone,businessPhones";
    
    // Start with the first page
    let mut url = format!(
        "https://graph.microsoft.com/v1.0/users?$select={}",
        urlencoding::encode(select_fields)
    );

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

    // Only update core fields, preserve role/pronouns/avatars
    let user_update = crate::models::UserUpdate {
        name: if updated_name != &user.name { Some(updated_name.clone()) } else { None },
        email: if updated_email != &user.email { Some(updated_email.clone()) } else { None },
        role: None, // Don't change role during sync
        pronouns: None, // Preserve pronouns
        avatar_url: None, // Preserve avatar
        banner_url: None, // Preserve banner
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
    if let Ok(avatar_url) = sync_user_profile_photo(&client, access_token, ms_user, &user.uuid).await {
        if let Err(e) = update_user_avatar_by_id(conn, user.id, avatar_url).await {
            println!("Warning: Failed to update avatar for user {}: {}", user.name, e);
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
        };

        user_repo::update_user(existing_user.id, user_update, conn)
            .map_err(|e| format!("Failed to update user name: {}", e))?;
    }

    // Sync profile photo
    let client = reqwest::Client::new();
    if let Ok(avatar_url) = sync_user_profile_photo(&client, access_token, ms_user, &existing_user.uuid).await {
        if let Err(e) = update_user_avatar_by_id(conn, existing_user.id, avatar_url).await {
            println!("Warning: Failed to update avatar for user {}: {}", existing_user.name, e);
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
    if let Ok(avatar_url) = sync_user_profile_photo(&client, access_token, ms_user, &user_uuid).await {
        if let Err(e) = update_user_avatar_by_id(conn, created_user.id, avatar_url).await {
            println!("Warning: Failed to update avatar for user {}: {}", name, e);
        }
    } else {
        println!("No profile photo available for user: {}", name);
    }

    println!("Created new user: {} ({})", name, email);
    stats.new_users_created += 1;
    Ok(())
}

/// Sync devices from Microsoft Graph
async fn sync_devices(
    _conn: &mut DbConnection,
    _provider_id: i32,
) -> SyncProgress {
    // TODO: Implement actual device sync logic
    SyncProgress {
        entity: "devices".to_string(),
        processed: 0,
        total: 0,
        status: "completed".to_string(),
        errors: vec!["Device sync not yet implemented".to_string()],
    }
}

/// Sync groups from Microsoft Graph
async fn sync_groups(
    _conn: &mut DbConnection,
    _provider_id: i32,
) -> SyncProgress {
    // TODO: Implement actual group sync logic
    SyncProgress {
        entity: "groups".to_string(),
        processed: 0,
        total: 0,
        status: "completed".to_string(),
        errors: vec!["Group sync not yet implemented".to_string()],
    }
}

/// Fetch and save user profile photo from Microsoft Graph
async fn sync_user_profile_photo(
    client: &reqwest::Client,
    access_token: &str,
    user: &MicrosoftGraphUser,
    local_user_uuid: &str,
) -> Result<Option<String>, String> {
    println!("Fetching 64x64 profile photo for user: {}", user.user_principal_name);

    // Try to get the user's profile photo in 64x64 size specifically
    let photo_url = format!("https://graph.microsoft.com/v1.0/users/{}/photos/64x64/$value", user.id);
    
    let photo_response = client
        .get(&photo_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch 64x64 profile photo: {}", e))?;

    if !photo_response.status().is_success() {
        // If 64x64 is not available, try the default size as fallback
        if photo_response.status().as_u16() == 404 {
            println!("64x64 profile photo not found for user: {}, trying default size", user.user_principal_name);
            return sync_user_profile_photo_fallback(client, access_token, user, local_user_uuid).await;
        } else {
            return Err(format!("Failed to fetch profile photo, status: {}", photo_response.status()));
        }
    }

    // Get the photo data
    let photo_bytes = photo_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read photo data: {}", e))?;

    if photo_bytes.is_empty() {
        println!("Empty 64x64 profile photo for user: {}", user.user_principal_name);
        return Ok(None);
    }

    println!("Successfully downloaded 64x64 profile photo ({} bytes) for user: {}", photo_bytes.len(), user.user_principal_name);

    // Save the photo to the filesystem
    save_profile_photo_to_disk(&photo_bytes, local_user_uuid, "64x64").await
}

/// Fallback function to get profile photo in default size if 64x64 is not available
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

    if !photo_response.status().is_success() {
        // User likely doesn't have a profile photo
        if photo_response.status().as_u16() == 404 {
            println!("No profile photo found for user: {}", user.user_principal_name);
            return Ok(None);
        } else {
            return Err(format!("Failed to fetch profile photo, status: {}", photo_response.status()));
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

    // Generate a unique filename for the photo with size label
    let file_extension = "jpg"; // Microsoft Graph typically returns JPEG photos
    let filename = format!("{}_{}_{}x{}.{}", local_user_uuid, Uuid::new_v4(), size_label, size_label, file_extension);
    let file_path = format!("{}/{}", avatar_dir, filename);

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

    println!("Saved profile photo for user to: {}", file_path);

    // Return the URL path for the database
    let avatar_url = format!("/{}", file_path);
    Ok(Some(avatar_url))
}

/// Update user avatar URL in the database
async fn update_user_avatar_by_id(
    conn: &mut DbConnection,
    user_id: i32,
    avatar_url: Option<String>,
) -> Result<(), String> {
    if let Some(url) = avatar_url {
        let user_update = crate::models::UserUpdate {
            name: None,
            email: None,
            role: None,
            pronouns: None,
            avatar_url: Some(url.clone()),
            banner_url: None,
        };

        user_repo::update_user(user_id, user_update, conn)
            .map_err(|e| format!("Failed to update user avatar: {}", e))?;

        println!("Updated avatar URL for user ID {}: {}", user_id, url);
    }

    Ok(())
}

/// Sync profile photos for all Microsoft users
pub async fn sync_profile_photos(
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

    // Sync profile photos for existing Microsoft users
    match sync_existing_user_photos(&mut conn, provider.id).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "message": format!("Profile photo sync failed: {}", error)
        }))
    }
} 