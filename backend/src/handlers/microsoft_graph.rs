use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde_json::json;
use serde::Deserialize;
use urlencoding;

use crate::db::Pool;
use crate::handlers::auth::validate_token_internal;
use crate::repository::auth_providers as auth_provider_repo;
use crate::config_utils;

// Structure for Microsoft Graph API requests
#[derive(Deserialize, Debug)]
pub struct MicrosoftGraphRequest {
    pub provider_id: Option<i32>,
    pub endpoint: String,
    pub method: Option<String>,
    pub body: Option<serde_json::Value>,
    pub query_params: Option<serde_json::Value>,
    pub headers: Option<serde_json::Value>,
}

// Microsoft Graph API request handler
pub async fn process_graph_request(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    request_data: web::Json<MicrosoftGraphRequest>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get user info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Get the provider_id from the request or use the default Microsoft provider
    let provider_id_val = match request_data.provider_id {
        Some(id) => id,
        None => {
            // Find the default Microsoft provider
            match auth_provider_repo::get_default_provider_by_type("microsoft", &mut conn) {
                Ok(provider) => provider.id,
                Err(e) => {
                    if let diesel::result::Error::NotFound = e {
                        return HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "No Microsoft authentication provider configured"
                        }));
                    } else {
                        eprintln!("Error getting default Microsoft provider: {:?}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Failed to retrieve Microsoft provider"
                        }));
                    }
                }
            }
        }
    };

    // Get the provider
    let provider = match auth_provider_repo::get_provider_by_id(provider_id_val, &mut conn) {
        Ok(p) => {
            if p.provider_type != "microsoft" {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "This endpoint only supports Microsoft Graph API requests"
                }));
            }
            p
        },
        Err(e) => {
            if let diesel::result::Error::NotFound = e {
                return HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "Authentication provider not found"
                }));
            } else {
                eprintln!("Error getting auth provider {}: {:?}", provider_id_val, e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to retrieve authentication provider"
                }));
            }
        }
    };

    // Check if the provider is enabled
    if !provider.enabled {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "The Microsoft authentication provider is not enabled"
        }));
    }

    // Get required configuration values from environment variables
    let client_id = match config_utils::get_microsoft_client_id() {
        Ok(val) => val,
        Err(e) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Microsoft configuration error: {}", e)
        })),
    };

    let tenant_id = match config_utils::get_microsoft_tenant_id() {
        Ok(val) => val,
        Err(e) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Microsoft configuration error: {}", e)
        })),
    };

    let client_secret = match config_utils::get_microsoft_client_secret() {
        Ok(val) => val,
        Err(e) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Microsoft configuration error: {}", e)
        })),
    };

    // Get an access token
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "client_credentials"),
        ("scope", "https://graph.microsoft.com/.default"),
    ];

    // Make the token request
    let client = reqwest::Client::new();
    let token_response = match client
        .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
        .form(&params)
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(token_data) => {
                    if token_data.get("access_token").is_none() {
                        return HttpResponse::BadRequest().json(json!({
                            "status": "error",
                            "message": "Failed to obtain access token",
                            "details": token_data.get("error_description").and_then(|v| v.as_str()).unwrap_or("Unknown error")
                        }));
                    }
                    token_data
                },
                Err(e) => {
                    eprintln!("Error parsing token response: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to parse Microsoft authentication response"
                    }));
                }
            }
        },
        Err(e) => {
            eprintln!("Error getting Microsoft access token: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to get Microsoft access token"
            }));
        }
    };

    let access_token = token_response["access_token"].as_str().unwrap();

    // Build the Microsoft Graph API request
    let endpoint = &request_data.endpoint;
    let method = request_data.method.as_deref().unwrap_or("GET");
    
    // Build the URL with query parameters if provided
    let mut url = format!("https://graph.microsoft.com/v1.0{}", endpoint);
    
    if let Some(query) = &request_data.query_params {
        if let Some(obj) = query.as_object() {
            let mut query_string = String::new();
            
            for (key, value) in obj {
                if !query_string.is_empty() {
                    query_string.push('&');
                } else {
                    query_string.push('?');
                }
                
                if let Some(str_val) = value.as_str() {
                    query_string.push_str(&format!("{}={}", urlencoding::encode(key), urlencoding::encode(str_val)));
                } else {
                    query_string.push_str(&format!("{}={}", urlencoding::encode(key), urlencoding::encode(&value.to_string())));
                }
            }
            
            url.push_str(&query_string);
        }
    }

    // Create the request based on the method
    let mut request_builder = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => {
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("Unsupported HTTP method: {}", method)
            }));
        }
    };

    // Add the authorization header
    request_builder = request_builder.header("Authorization", format!("Bearer {}", access_token));
    
    // Add custom headers if provided
    if let Some(headers) = &request_data.headers {
        if let Some(obj) = headers.as_object() {
            for (key, value) in obj {
                if let Some(str_val) = value.as_str() {
                    request_builder = request_builder.header(key, str_val);
                }
            }
        }
    }

    // Add the request body if provided for non-GET/DELETE methods
    if method != "GET" && method != "DELETE" {
        if let Some(body) = &request_data.body {
            request_builder = request_builder.json(body);
        }
    }

    // Execute the request
    match request_builder.send().await {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    HttpResponse::build(status).json(json!({
                        "status": if status.is_success() { "success" } else { "error" },
                        "data": data
                    }))
                },
                Err(e) => {
                    eprintln!("Error parsing Microsoft Graph response: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to parse Microsoft Graph response"
                    }))
                }
            }
        },
        Err(e) => {
            eprintln!("Error sending Microsoft Graph request: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to send Microsoft Graph request"
            }))
        }
    }
}

// Helper endpoints for common Graph API operations
pub async fn get_graph_users(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Convert the query parameters to what Microsoft Graph needs, only including non-empty values
    let mut query_params = serde_json::Map::new();
    
    if let Some(filter) = query.get("filter") {
        if !filter.is_empty() {
            query_params.insert("$filter".to_string(), serde_json::Value::String(filter.clone()));
        }
    }
    
    if let Some(select) = query.get("select") {
        if !select.is_empty() {
            query_params.insert("$select".to_string(), serde_json::Value::String(select.clone()));
        }
    }
    
    if let Some(top) = query.get("top") {
        if !top.is_empty() {
            query_params.insert("$top".to_string(), serde_json::Value::String(top.clone()));
        }
    }
    
    // Handle providerId separately as it's not a Graph API parameter
    let provider_id = query.get("providerId").and_then(|id| id.parse::<i32>().ok());

    // Create a Graph request
    let graph_request = MicrosoftGraphRequest {
        provider_id,
        endpoint: "/users".to_string(),
        method: Some("GET".to_string()),
        body: None,
        query_params: if query_params.is_empty() { None } else { Some(serde_json::Value::Object(query_params)) },
        headers: None,
    };

    // Process the request
    process_graph_request(db_pool, auth, web::Json(graph_request)).await
}

pub async fn get_graph_devices(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Convert the query parameters to what Microsoft Graph needs, only including non-empty values
    let mut query_params = serde_json::Map::new();
    
    if let Some(filter) = query.get("filter") {
        if !filter.is_empty() {
            query_params.insert("$filter".to_string(), serde_json::Value::String(filter.clone()));
        }
    }
    
    if let Some(select) = query.get("select") {
        if !select.is_empty() {
            query_params.insert("$select".to_string(), serde_json::Value::String(select.clone()));
        }
    }
    
    if let Some(top) = query.get("top") {
        if !top.is_empty() {
            query_params.insert("$top".to_string(), serde_json::Value::String(top.clone()));
        }
    }
    
    // Handle providerId separately as it's not a Graph API parameter
    let provider_id = query.get("providerId").and_then(|id| id.parse::<i32>().ok());

    // Create a Graph request
    let graph_request = MicrosoftGraphRequest {
        provider_id,
        endpoint: "/devices".to_string(),
        method: Some("GET".to_string()),
        body: None,
        query_params: if query_params.is_empty() { None } else { Some(serde_json::Value::Object(query_params)) },
        headers: None,
    };

    // Process the request
    process_graph_request(db_pool, auth, web::Json(graph_request)).await
}

pub async fn get_graph_groups(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Convert the query parameters to what Microsoft Graph needs, only including non-empty values
    let mut query_params = serde_json::Map::new();
    
    if let Some(filter) = query.get("filter") {
        if !filter.is_empty() {
            query_params.insert("$filter".to_string(), serde_json::Value::String(filter.clone()));
        }
    }
    
    if let Some(select) = query.get("select") {
        if !select.is_empty() {
            query_params.insert("$select".to_string(), serde_json::Value::String(select.clone()));
        }
    }
    
    if let Some(top) = query.get("top") {
        if !top.is_empty() {
            query_params.insert("$top".to_string(), serde_json::Value::String(top.clone()));
        }
    }
    
    // Handle providerId separately as it's not a Graph API parameter
    let provider_id = query.get("providerId").and_then(|id| id.parse::<i32>().ok());

    // Create a Graph request
    let graph_request = MicrosoftGraphRequest {
        provider_id,
        endpoint: "/groups".to_string(),
        method: Some("GET".to_string()),
        body: None,
        query_params: if query_params.is_empty() { None } else { Some(serde_json::Value::Object(query_params)) },
        headers: None,
    };

    // Process the request
    process_graph_request(db_pool, auth, web::Json(graph_request)).await
}

pub async fn get_graph_directory_objects(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Convert the query parameters to what Microsoft Graph needs, only including non-empty values
    let mut query_params = serde_json::Map::new();
    
    if let Some(filter) = query.get("filter") {
        if !filter.is_empty() {
            query_params.insert("$filter".to_string(), serde_json::Value::String(filter.clone()));
        }
    }
    
    if let Some(select) = query.get("select") {
        if !select.is_empty() {
            query_params.insert("$select".to_string(), serde_json::Value::String(select.clone()));
        }
    }
    
    if let Some(top) = query.get("top") {
        if !top.is_empty() {
            query_params.insert("$top".to_string(), serde_json::Value::String(top.clone()));
        }
    }
    
    // Handle providerId separately as it's not a Graph API parameter
    let provider_id = query.get("providerId").and_then(|id| id.parse::<i32>().ok());

    // Create a Graph request for directory objects
    let graph_request = MicrosoftGraphRequest {
        provider_id,
        endpoint: "/directoryObjects".to_string(),
        method: Some("GET".to_string()),
        body: None,
        query_params: if query_params.is_empty() { None } else { Some(serde_json::Value::Object(query_params)) },
        headers: None,
    };

    // Process the request
    process_graph_request(db_pool, auth, web::Json(graph_request)).await
} 