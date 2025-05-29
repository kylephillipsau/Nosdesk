use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use urlencoding;
use querystring;
use serde::Deserialize;

use crate::db::{Pool, DbConnection};
use crate::handlers::auth::{validate_token_internal, JWT_SECRET};
use crate::models::{
    AuthProvider, AuthProviderConfigRequest, ConfigItem, NewAuthProvider,
    AuthProviderUpdate, OAuthRequest, OAuthExchangeRequest,
    OAuthState
};
use crate::repository::auth_providers as auth_provider_repo;
use crate::repository::user_auth_identities;
use crate::config_utils;

// Structure for OAuth logout requests
#[derive(Deserialize, Debug)]
pub struct OAuthLogoutRequest {
    pub provider_type: String,
    pub redirect_uri: String,
}

// Get all authentication providers (admin only)
pub async fn get_auth_providers(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can manage authentication providers"
        }));
    }

    // Get all providers with their configurations
    match auth_provider_repo::get_all_providers(&mut conn) {
        Ok(providers) => HttpResponse::Ok().json(providers),
        Err(e) => {
            eprintln!("Error getting auth providers: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve authentication providers"
            }))
        }
    }
}

// Get enabled authentication providers (for login page)
pub async fn get_enabled_auth_providers(
    db_pool: web::Data<Pool>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Get enabled providers
    match auth_provider_repo::get_enabled_providers(&mut conn) {
        Ok(providers) => {
            // Convert to simple response format (no secrets or configurations)
            let response = providers.into_iter().map(|p| json!({
                "id": p.id,
                "provider_type": p.provider_type,
                "name": p.name,
                "is_default": p.is_default
            })).collect::<Vec<_>>();
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            eprintln!("Error getting enabled auth providers: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve authentication providers"
            }))
        }
    }
}

// Get a specific authentication provider with its configuration (admin only)
pub async fn get_auth_provider(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    path: web::Path<i32>,
) -> impl Responder {
    let provider_id = path.into_inner();
    
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can manage authentication providers"
        }));
    }

    // Get the provider with its configuration
    match auth_provider_repo::get_provider_by_id(provider_id, &mut conn) {
        Ok(provider) => HttpResponse::Ok().json(provider),
        Err(e) => {
            if let diesel::result::Error::NotFound = e {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "Authentication provider not found"
                }))
            } else {
                eprintln!("Error getting auth provider {}: {:?}", provider_id, e);
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to retrieve authentication provider"
                }))
            }
        }
    }
}

// Create a new authentication provider (admin only)
pub async fn create_auth_provider(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    provider_data: web::Json<NewAuthProvider>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can manage authentication providers"
        }));
    }

    // Create the provider
    match auth_provider_repo::create_provider(provider_data.into_inner(), &mut conn) {
        Ok(provider) => HttpResponse::Created().json(provider),
        Err(e) => {
            eprintln!("Error creating auth provider: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to create authentication provider"
            }))
        }
    }
}

// Update an authentication provider (admin only)
pub async fn update_auth_provider(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    path: web::Path<i32>,
    provider_data: web::Json<AuthProviderUpdate>,
) -> impl Responder {
    let provider_id = path.into_inner();
    
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can manage authentication providers"
        }));
    }

    // If we're setting this provider as default, handle it with a transaction
    if provider_data.is_default.unwrap_or(false) {
        match auth_provider_repo::set_default_provider(provider_id, &mut conn) {
            Ok(_) => {
                // Continue with the rest of the update
            },
            Err(e) => {
                eprintln!("Error setting default provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to set default provider"
                }));
            }
        }
    }

    // Update the provider
    match auth_provider_repo::update_provider(provider_id, provider_data.into_inner(), &mut conn) {
        Ok(provider) => HttpResponse::Ok().json(provider),
        Err(e) => {
            if let diesel::result::Error::NotFound = e {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "Authentication provider not found"
                }))
            } else {
                eprintln!("Error updating auth provider {}: {:?}", provider_id, e);
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to update authentication provider"
                }))
            }
        }
    }
}

// Delete an authentication provider (admin only)
pub async fn delete_auth_provider(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    path: web::Path<i32>,
) -> impl Responder {
    let provider_id = path.into_inner();
    
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can manage authentication providers"
        }));
    }

    // Don't allow deleting the local provider
    match auth_provider_repo::get_provider_by_id(provider_id, &mut conn) {
        Ok(provider) => {
            if provider.provider_type == "local" {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "The local authentication provider cannot be deleted"
                }));
            }
        },
        Err(e) => {
            if let diesel::result::Error::NotFound = e {
                return HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": "Authentication provider not found"
                }));
            } else {
                eprintln!("Error getting auth provider {}: {:?}", provider_id, e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to retrieve authentication provider"
                }));
            }
        }
    }

    // Delete the provider
    match auth_provider_repo::delete_provider(provider_id, &mut conn) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            eprintln!("Error deleting auth provider {}: {:?}", provider_id, e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to delete authentication provider"
            }))
        }
    }
}

// Update provider configuration (admin only)
pub async fn update_auth_provider_config(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    config_request: web::Json<AuthProviderConfigRequest>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can manage authentication providers"
        }));
    }

    let provider_id = config_request.provider_id;
    
    // Get the provider type
    let provider = match auth_provider_repo::get_provider_by_id(provider_id, &mut conn) {
        Ok(p) => p,
        Err(_) => return HttpResponse::NotFound().json(json!({
                "status": "error",
            "message": "Provider not found"
        }))
    };

    // Since we've removed the auth_provider_configs table, all configuration
    // is now managed through environment variables for Microsoft providers
    if provider.provider_type == "microsoft" {
        return HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Microsoft provider configuration is managed by environment variables on the server. Please set MICROSOFT_CLIENT_ID, MICROSOFT_TENANT_ID, MICROSOFT_CLIENT_SECRET, and MICROSOFT_REDIRECT_URI in your server environment.",
            "provider": provider
        }));
    } else {
        return HttpResponse::BadRequest().json(json!({
                "status": "error",
            "message": "Configuration management for this provider type is not currently supported"
        }));
    }
}

// Generate OAuth authorization URL
pub async fn oauth_authorize(
    db_pool: web::Data<Pool>,
    oauth_request: web::Json<OAuthRequest>,
    auth: Option<BearerAuth>, // Make auth optional, it's only required for user connection
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Check if this is a user connection request
    let is_user_connection = oauth_request.user_connection.unwrap_or(false);

    // Validate user token if this is a connection request
    let user_uuid = if is_user_connection {
        // Auth token is required for connection requests
        let auth_token = match &auth {
            Some(token) => token,
            None => return HttpResponse::Unauthorized().json(json!({
                "status": "error", 
                "message": "Authentication required for connecting accounts"
            })),
        };

        // Validate the token and get user info
        let claims = match validate_token_internal(auth_token, &mut conn).await {
            Ok(claims) => claims,
            Err(_) => return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid or expired token"
            })),
        };

        // Return the user UUID from claims
        Some(claims.sub)
    } else {
        None
    };

    let provider_type = &oauth_request.provider_type;
    
    // Get the provider by type
    let provider = match auth_provider_repo::get_provider_by_type(provider_type, &mut conn) {
        Ok(p) => {
            if !p.enabled {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": format!("{} authentication is not enabled", p.name)
                }));
            }
            p
        },
        Err(e) => {
            if let diesel::result::Error::NotFound = e {
                return HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": format!("{} authentication provider not found", provider_type)
                }));
            } else {
                eprintln!("Error getting auth provider {}: {:?}", provider_type, e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to retrieve authentication provider"
                }));
            }
        }
    };

    // For Microsoft Entra, generate the authorization URL
    if provider.provider_type == "microsoft" {
        // Get the provider configuration from environment variables
        let client_id = match config_utils::get_microsoft_client_id() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting client_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        let tenant_id = match config_utils::get_microsoft_tenant_id() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting tenant_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        let redirect_uri_config = match config_utils::get_microsoft_redirect_uri() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting redirect_uri for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        // Generate a JWT state token
        let state = match create_oauth_state(&provider.provider_type, oauth_request.redirect_uri.clone(), oauth_request.user_connection) {
            Ok(token) => token,
            Err(e) => {
                eprintln!("Error creating OAuth state token: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to initiate authentication flow"
                }));
            }
        };
        
        // Create the authorization URL
        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=User.Read&state={}",
            tenant_id, client_id, redirect_uri_config, state
        );
        
        HttpResponse::Ok().json(json!({
            "auth_url": auth_url,
            "state": state
        }))
    } else {
        HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("{} authentication is not implemented", provider.name)
        }))
    }
}

// Handle OAuth callback and token exchange
pub async fn oauth_callback(
    db_pool: web::Data<Pool>,
    query: web::Query<OAuthExchangeRequest>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Verify state parameter is present
    let state = match &query.state {
        Some(state) => state,
        None => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Missing state parameter"
        })),
    };

    // Verify code parameter is present
    let code = match &query.code {
        Some(code) => code,
        None => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Missing authorization code"
        })),
    };

    // Verify the state JWT
    let state_data = match verify_oauth_state(state) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error verifying OAuth state: {}", e);
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Invalid or expired state parameter"
            }));
        }
    };

    // Get the provider by type
    let provider_type = &state_data.provider_type;
    let provider = match auth_provider_repo::get_provider_by_type(provider_type, &mut conn) {
        Ok(p) => {
            if !p.enabled {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": format!("{} authentication is not enabled", p.name)
                }));
            }
            p
        },
        Err(e) => {
            eprintln!("Error getting provider in callback: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Authentication provider error"
            }));
        }
    };

    // Check if this is a user connection request (vs. a standard login)
    let is_connection = state_data.user_connection.unwrap_or(false);

    // Process based on provider type
    if provider.provider_type == "microsoft" {
        // Get the provider configuration from environment variables
        let client_id = match config_utils::get_microsoft_client_id() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting client_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        let tenant_id = match config_utils::get_microsoft_tenant_id() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting tenant_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        let client_secret = match config_utils::get_microsoft_client_secret() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting client_secret for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        let redirect_uri_config = match config_utils::get_microsoft_redirect_uri() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting redirect_uri for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        // Exchange authorization code for an access token
        let token_result = exchange_microsoft_code_for_token(&provider, code, &mut conn).await;
        
        match token_result {
            Ok((access_token, _refresh_token)) => {
                // Get user info from Microsoft
                let user_info = match get_microsoft_user_info(&access_token).await {
                    Ok(info) => info,
                    Err(e) => {
                        eprintln!("Error getting Microsoft user info: {:?}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Failed to get user information from Microsoft"
                        }));
                    }
                };
                
                // Extract unique identifier for Microsoft (object ID)
                let provider_user_id = match user_info.get("id").and_then(|id| id.as_str()) {
                    Some(id) => id.to_string(),
                    None => {
                        eprintln!("No ID found in Microsoft user info");
                        return HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Invalid user information from Microsoft"
                        }));
                    }
                };

                // Extract email from user info
                let email = match user_info.get("mail")
                    .or_else(|| user_info.get("userPrincipalName"))
                    .and_then(|e| e.as_str()) {
                    Some(email) => email.to_string(),
                    None => {
                        eprintln!("No email found in Microsoft user info");
                        return HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Invalid user information from Microsoft (no email)"
                        }));
                    }
                };

                // Handle account connection vs normal login
                if is_connection {
                    // This is a connection request - check if this identity is already linked to another account
                    match user_auth_identities::find_user_by_identity(&provider.provider_type, &provider_user_id, &mut conn) {
                        Ok(Some(_)) => {
                            return HttpResponse::BadRequest().json(json!({
                                "status": "error",
                                "message": "This Microsoft account is already connected to another user account"
                            }));
                        },
                        Ok(None) => {
                            // Identity not yet linked to any account, we can proceed
                        },
                        Err(e) => {
                            eprintln!("Error checking existing identity: {:?}", e);
                            return HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": "Failed to verify Microsoft account status"
                            }));
                        }
                    }
                    
                    // Extract user UUID from the redirect URL params if present
                    // For added security, we get the UUID from the query string in redirect_uri if provided
                    let user_uuid_param = if state_data.redirect_uri.contains('?') {
                        let query_params = state_data.redirect_uri.split('?').nth(1).unwrap_or("");
                        let params = querystring::querify(query_params);
                        params.iter()
                            .find(|(k, _)| *k == "user_uuid")
                            .map(|(_, v)| v.to_string())
                    } else {
                        None
                    };
                    
                    // Get user UUID from SessionStorage on client side if not in URL
                    // SessionStorage on the frontend should contain the authRedirect with the user's path
                    
                    // Add the identity to the user - for now we use a hardcoded false value
                    // but in a real implementation you'd get the UUID from auth token
                    let user_uuid = match user_uuid_param {
                        Some(uuid) => uuid,
                        None => {
                            // If not explicit in URL params, then the user should be authenticated,
                            // which means we need this from request headers (auth token)
                            let redirect_parts: Vec<&str> = state_data.redirect_uri.split('?').collect();
                            let redirect_path = redirect_parts[0];
                            
                            // Error case - can't determine user
                            let error_url = format!("{}?auth_error={}", 
                                redirect_path,
                                "Could not determine user account for connection");
                            
                            return HttpResponse::Found()
                                .append_header(("Location", error_url))
                                .finish();
                        }
                    };
                    
                    // Add the identity to the user account
                    match add_oauth_identity_to_user(&user_uuid, &user_info, &provider, &mut conn).await {
                        Ok(_) => {
                            // Successful connection
                            let redirect_parts: Vec<&str> = state_data.redirect_uri.split('?').collect();
                            let redirect_path = redirect_parts[0];
                            let success_url = format!("{}?auth_success=true", redirect_path);
                            
                            // Redirect to success page
                            HttpResponse::Found()
                                .append_header(("Location", success_url))
                                .finish()
                        },
                        Err(e) => {
                            eprintln!("Error connecting account: {}", e);
                            
                            // Error connecting
                            let redirect_parts: Vec<&str> = state_data.redirect_uri.split('?').collect();
                            let redirect_path = redirect_parts[0];
                            let error_url = format!("{}?auth_error={}", 
                                redirect_path,
                                urlencoding::encode(&format!("Failed to connect account: {}", e)));
                            
                            HttpResponse::Found()
                                .append_header(("Location", error_url))
                                .finish()
                        }
                    }
                } else {
                    // Regular login/signup flow
                    // Find or create user based on OAuth identity
                    let user_result = find_or_create_oauth_user(&user_info, &provider, &mut conn).await;
                    
                    match user_result {
                        Ok(user) => {
                            // Generate JWT token for our application
                            let token = match generate_app_jwt_token(&user) {
                                Ok(token) => token,
                                Err(e) => {
                                    eprintln!("Error generating app JWT token: {:?}", e);
                                    return HttpResponse::InternalServerError().json(json!({
                                        "status": "error",
                                        "message": "Failed to generate authentication token"
                                    }));
                                }
                            };
                            
                            // Instead of redirecting, return a JSON response with the token and user info
                            HttpResponse::Ok().json(json!({
                                "token": token,
                                "user": crate::models::UserResponse::from(user),
                                "redirect": state_data.redirect_uri
                            }))
                        },
                        Err(e) => {
                            eprintln!("Error finding/creating user: {:?}", e);
                            HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": "Failed to authenticate user"
                            }))
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Error exchanging code for token: {:?}", e);
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to authenticate with Microsoft"
                }))
            }
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("{} authentication callback is not implemented", provider.name)
        }))
    }
}

// Handle OAuth logout request
pub async fn oauth_logout(
    db_pool: web::Data<Pool>,
    logout_request: web::Json<OAuthLogoutRequest>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    let provider_type = &logout_request.provider_type;
    
    // Get the provider by type
    let provider = match auth_provider_repo::get_provider_by_type(provider_type, &mut conn) {
        Ok(p) => {
            if !p.enabled {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": format!("{} authentication is not enabled", p.name)
                }));
            }
            p
        },
        Err(e) => {
            if let diesel::result::Error::NotFound = e {
                return HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": format!("{} authentication provider not found", provider_type)
                }));
            } else {
                eprintln!("Error getting auth provider {}: {:?}", provider_type, e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to retrieve authentication provider"
                }));
            }
        }
    };

    // For Microsoft Entra, generate the logout URL
    if provider.provider_type == "microsoft" {
        // Get the tenant ID from environment variables
        let tenant_id = match config_utils::get_microsoft_tenant_id() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting tenant_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        // URL encode the redirect URI
        let encoded_redirect = urlencoding::encode(&logout_request.redirect_uri);
        
        // Create the logout URL
        let logout_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/logout?post_logout_redirect_uri={}",
            tenant_id, encoded_redirect
        );
        
        HttpResponse::Ok().json(json!({
            "logout_url": logout_url
        }))
    } else {
        HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("{} logout is not implemented", provider.name)
        }))
    }
}

// JWT State Management

// Create a signed state JWT for OAuth flow
fn create_oauth_state(provider_type: &str, redirect_uri: Option<String>, user_connection: Option<bool>) -> Result<String, String> {
    // Get the JWT secret from environment or configuration
    let secret = JWT_SECRET.clone();
    
    // Create expiration timestamp (10 minutes from now)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let exp = now + (10 * 60); // 10 minutes
    
    // Create claims
    let state = format!("{:x}", rand::random::<u128>());
    let claims = OAuthState {
        state: state.clone(),
        redirect_uri: redirect_uri.unwrap_or_else(|| "/".to_string()),
        provider_type: provider_type.to_string(),
        exp,
        user_connection,
    };
    
    // Create the token
    match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(token) => Ok(token),
        Err(e) => Err(format!("Failed to create state JWT: {}", e)),
    }
}

// Verify and decode the state JWT
fn verify_oauth_state(token: &str) -> Result<OAuthState, String> {
    // Get the JWT secret from environment or configuration
    let secret = JWT_SECRET.clone();
    
    // Verify the token
    match jsonwebtoken::decode::<OAuthState>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(data) => Ok(data.claims),
        Err(e) => Err(format!("Invalid state JWT: {}", e)),
    }
}

// Helper function to exchange Microsoft code for token
async fn exchange_microsoft_code_for_token(
    provider: &AuthProvider,
    code: &str,
    conn: &mut DbConnection,
) -> Result<(String, Option<String>), String> {
    // Get provider configuration from environment variables
    let client_id = match config_utils::get_microsoft_client_id() {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to get client_id: {}", e)),
    };
    
    let tenant_id = match config_utils::get_microsoft_tenant_id() {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to get tenant_id: {}", e)),
    };
    
    let client_secret = match config_utils::get_microsoft_client_secret() {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to get client_secret: {}", e)),
    };
    
    let redirect_uri_config = match config_utils::get_microsoft_redirect_uri() {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to get redirect_uri: {}", e)),
    };
    
    // Prepare the token request
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", code),
        ("redirect_uri", redirect_uri_config.as_str()),
        ("grant_type", "authorization_code"),
    ];
    
    // Make the token request
    let client = reqwest::Client::new();
    let res = match client
        .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
        .form(&params)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => return Err(format!("Failed to send token request: {}", e)),
    };
    
    // Parse the response
    let token_response = match res.json::<serde_json::Value>().await {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to parse token response: {}", e)),
    };
    
    // Extract tokens
    let access_token = match token_response.get("access_token") {
        Some(token) => match token.as_str() {
            Some(t) => t.to_string(),
            None => return Err("Invalid access token format".to_string()),
        },
        None => return Err("No access token in response".to_string()),
    };
    
    let refresh_token = token_response.get("refresh_token").and_then(|t| t.as_str()).map(|s| s.to_string());
    
    Ok((access_token, refresh_token))
}

// Helper function to get user info from Microsoft Graph API
async fn get_microsoft_user_info(access_token: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let res = match client
        .get("https://graph.microsoft.com/v1.0/me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => return Err(format!("Failed to get user info: {}", e)),
    };
    
    match res.json::<serde_json::Value>().await {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("Failed to parse user info response: {}", e)),
    }
}

// Helper function to find or create a user from OAuth profile
async fn find_or_create_oauth_user(
    user_info: &serde_json::Value,
    provider: &AuthProvider,
    conn: &mut DbConnection,
) -> Result<crate::models::User, String> {
    // Extract email from user info
    let email = match user_info.get("mail").or_else(|| user_info.get("userPrincipalName")) {
        Some(email) => match email.as_str() {
            Some(e) => e.to_string(),
            None => return Err("Invalid email format".to_string()),
        },
        None => return Err("No email in user info".to_string()),
    };
    
    // Extract name from user info
    let name = match user_info.get("displayName") {
        Some(name) => match name.as_str() {
            Some(n) => n.to_string(),
            None => return Err("Invalid name format".to_string()),
        },
        None => return Err("No name in user info".to_string()),
    };
    
    // Extract unique identifier for Microsoft (object ID)
    let provider_user_id = match user_info.get("id") {
        Some(id) => match id.as_str() {
            Some(i) => i.to_string(),
            None => return Err("Invalid id format".to_string()),
        },
        None => return Err("No id in user info".to_string()),
    };
    
    use crate::models::{User, NewUserAuthIdentity};
    
    // First try to find the user by their external identity
    match user_auth_identities::find_user_by_identity(&provider.provider_type, &provider_user_id, conn) {
        Ok(Some(user_id)) => {
            // User found by identity, return the user
            match crate::repository::get_user_by_id(user_id, conn) {
                Ok(user) => return Ok(user),
                Err(e) => return Err(format!("Error retrieving user: {:?}", e)),
            }
        },
        Ok(None) => {
            // No identity found, look for the user by email as a fallback
            match crate::repository::get_user_by_email(&email, conn) {
                Ok(user) => {
                    // User found by email, create an identity for them
                    let new_identity = NewUserAuthIdentity {
                        user_id: user.id,
                        auth_provider_id: provider.id,
                        provider_user_id: provider_user_id.clone(),
                        email: Some(email.clone()),
                        identity_data: Some(user_info.clone()),
                        password_hash: None, // No password for OAuth identities
                    };
                    
                    match user_auth_identities::create_identity(new_identity, conn) {
                        Ok(_) => {
                            // Identity created, return the user
                            return Ok(user);
                        },
                        Err(e) => {
                            eprintln!("Error creating user identity: {:?}", e);
                            // Continue to create a new user
                        }
                    }
                },
                Err(_) => {
                    // User not found by email, create a new one
                }
            }
        },
        Err(e) => {
            eprintln!("Error finding user by identity: {:?}", e);
            // Continue to create a new user
        }
    }
    
    // Create a new user
    use crate::models::{NewUser, UserRole};
    use bcrypt::hash;
    use uuid::Uuid;
    
    // Generate a secure random password for the user
    let random_password = format!("{:x}", rand::random::<u128>());
    let password_hash = match hash(&random_password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash.into_bytes(),
        Err(e) => return Err(format!("Failed to hash password: {}", e)),
    };
    
    let new_user = NewUser {
        uuid: Uuid::new_v4().to_string(),
        name,
        email: email.clone(),
        role: "user".to_string(), // Default to regular user
        pronouns: None,
        avatar_url: None,
        banner_url: None,
        avatar_thumb: None,
        microsoft_uuid: None, // OAuth users don't have Microsoft UUID initially
    };
    
    match crate::repository::create_user(new_user, conn) {
        Ok(user) => {
            // Create an identity for the new user
            let new_identity = NewUserAuthIdentity {
                user_id: user.id,
                auth_provider_id: provider.id,
                provider_user_id,
                email: Some(email),
                identity_data: Some(user_info.clone()),
                password_hash: Some(password_hash), // Add the password hash to the identity
            };
            
            match user_auth_identities::create_identity(new_identity, conn) {
                Ok(_) => Ok(user),
                Err(e) => Err(format!("User created but failed to create identity: {:?}", e)),
            }
        },
        Err(e) => Err(format!("Failed to create user: {:?}", e)),
    }
}

// Helper function to generate application JWT token
fn generate_app_jwt_token(user: &crate::models::User) -> Result<String, String> {
    use crate::models::Claims;
    
    // Get the JWT secret from environment or configuration
    let secret = JWT_SECRET.clone();
    
    // Generate JWT token
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        sub: user.uuid.clone(),
        name: user.name.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp: now + 24 * 60 * 60, // 24 hours from now
        iat: now,
    };

    match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(token) => Ok(token),
        Err(e) => Err(format!("Failed to create JWT: {}", e)),
    }
}

// Helper function to add an OAuth identity to an existing user
async fn add_oauth_identity_to_user(
    user_uuid: &str,
    user_info: &serde_json::Value, 
    provider: &AuthProvider,
    conn: &mut DbConnection
) -> Result<(), String> {
    // First find the user by UUID
    let user = match crate::repository::get_user_by_uuid(user_uuid, conn) {
        Ok(user) => user,
        Err(e) => return Err(format!("User not found: {:?}", e)),
    };
    
    // Extract unique identifier for Microsoft (object ID)
    let provider_user_id = match user_info.get("id") {
        Some(id) => match id.as_str() {
            Some(i) => i.to_string(),
            None => return Err("Invalid id format".to_string()),
        },
        None => return Err("No id in user info".to_string()),
    };
    
    // Extract email from user info (optional)
    let email = user_info.get("mail")
        .or_else(|| user_info.get("userPrincipalName"))
        .and_then(|e| e.as_str())
        .map(|e| e.to_string());
    
    // Create a new identity for the user
    let new_identity = crate::models::NewUserAuthIdentity {
        user_id: user.id,
        auth_provider_id: provider.id,
        provider_user_id,
        email,
        identity_data: Some(user_info.clone()),
        password_hash: None, // No password for OAuth identities
    };
    
    // Save the identity to the database
    match user_auth_identities::create_identity(new_identity, conn) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to create auth identity: {:?}", e)),
    }
}

// Direct endpoint for connecting a new authentication method to an existing user
pub async fn oauth_connect(
    db_pool: web::Data<Pool>,
    auth: BearerAuth, // Required auth token
    oauth_request: web::Json<OAuthRequest>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate token and get user info - this is required for connections
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Verify user exists
    let user = match crate::repository::get_user_by_uuid(&claims.sub, &mut conn) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Error finding user by UUID: {:?}", e);
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "User not found"
            }));
        }
    };

    let provider_type = &oauth_request.provider_type;
    
    // Get the provider by type
    let provider = match auth_provider_repo::get_provider_by_type(provider_type, &mut conn) {
        Ok(p) => {
            if !p.enabled {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": format!("{} authentication is not enabled", p.name)
                }));
            }
            p
        },
        Err(e) => {
            if let diesel::result::Error::NotFound = e {
                return HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": format!("{} authentication provider not found", provider_type)
                }));
            } else {
                eprintln!("Error getting auth provider {}: {:?}", provider_type, e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to retrieve authentication provider"
                }));
            }
        }
    };

    // For Microsoft Entra, generate the authorization URL
    if provider.provider_type == "microsoft" {
        // Get the provider configuration from environment variables
        let client_id = match config_utils::get_microsoft_client_id() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting client_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        let tenant_id = match config_utils::get_microsoft_tenant_id() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting tenant_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        let redirect_uri_config = match config_utils::get_microsoft_redirect_uri() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error getting redirect_uri for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Microsoft authentication is not properly configured: {}", e)
                }));
            }
        };
        
        // Prepare redirect URI with user UUID
        let mut actual_redirect_uri = oauth_request.redirect_uri.clone().unwrap_or_else(|| format!("/profile/settings"));
        if !actual_redirect_uri.contains("user_uuid=") {
            let separator = if actual_redirect_uri.contains('?') { "&" } else { "?" };
            actual_redirect_uri = format!("{}{}user_uuid={}", actual_redirect_uri, separator, user.uuid);
        }
        
        // Generate a JWT state token with user_connection=true
        let state = match create_oauth_state(&provider.provider_type, Some(actual_redirect_uri), Some(true)) {
            Ok(token) => token,
            Err(e) => {
                eprintln!("Error creating OAuth state token: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to initiate authentication flow"
                }));
            }
        };
        
        // Create the authorization URL
        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=User.Read&state={}",
            tenant_id, client_id, redirect_uri_config, state
        );
        
        HttpResponse::Ok().json(json!({
            "auth_url": auth_url,
            "state": state
        }))
    } else {
        HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("{} authentication is not implemented", provider.name)
        }))
    }
}

// Test Microsoft Entra configuration
pub async fn test_microsoft_config(
    db_pool: web::Data<Pool>,
    auth: BearerAuth,
    path: web::Path<i32>,
) -> impl Responder {
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Could not get database connection"
        })),
    };

    // Validate the token and get admin info
    let claims = match validate_token_internal(&auth, &mut conn).await {
        Ok(claims) => claims,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        })),
    };

    // Check if the user is an admin
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Only administrators can test authentication providers"
        }));
    }

    let provider_id = path.into_inner();

    // Get the provider
    let provider = match auth_provider_repo::get_provider_by_id(provider_id, &mut conn) {
        Ok(p) => {
            if p.provider_type != "microsoft" {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "This endpoint only supports testing Microsoft Entra configuration"
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
                eprintln!("Error getting auth provider {}: {:?}", provider_id, e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to retrieve authentication provider"
                }));
            }
        }
    };

    // Get required configuration values from environment variables
    let client_id = match config_utils::get_microsoft_client_id() {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("Missing client_id configuration: {}", e)
        })),
    };

    let tenant_id = match config_utils::get_microsoft_tenant_id() {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("Missing tenant_id configuration: {}", e)
        })),
    };

    let client_secret = match config_utils::get_microsoft_client_secret() {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("Missing client_secret configuration: {}", e)
        })),
    };

    let redirect_uri = match config_utils::get_microsoft_redirect_uri() {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("Missing redirect_uri configuration: {}", e)
        })),
    };

    // Test the configuration by attempting to get an access token
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "client_credentials"),
        ("scope", "https://graph.microsoft.com/.default"),
    ];

    // Make the token request
    let client = reqwest::Client::new();
    match client
        .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id))
        .form(&params)
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(token_response) => {
                    if token_response.get("access_token").is_some() {
                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "message": "Microsoft Entra configuration is valid",
                            "details": {
                                "client_id_valid": true,
                                "tenant_id_valid": true,
                                "client_secret_valid": true,
                                "redirect_uri_configured": true
                            }
                        }))
                    } else {
                        HttpResponse::BadRequest().json(json!({
                            "status": "error",
                            "message": "Invalid configuration",
                            "details": token_response.get("error_description").and_then(|v| v.as_str()).unwrap_or("Unknown error")
                        }))
                    }
                },
                Err(e) => {
                    eprintln!("Error parsing token response: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to parse Microsoft authentication response"
                    }))
                }
            }
        },
        Err(e) => {
            eprintln!("Error testing Microsoft configuration: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to test Microsoft configuration"
            }))
        }
    }
} 