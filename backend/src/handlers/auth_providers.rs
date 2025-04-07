use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::Pool;
use crate::handlers::auth::{validate_token_internal};
use crate::models::{
    AuthProvider, AuthProviderConfigRequest, ConfigItem, NewAuthProvider,
    NewAuthProviderConfig, AuthProviderUpdate, OAuthRequest, OAuthExchangeRequest,
    OAuthState
};
use crate::repository::auth_providers as auth_provider_repo;

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
    match auth_provider_repo::get_all_providers_with_configs(&mut conn) {
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
    match auth_provider_repo::get_provider_with_configs(provider_id, &mut conn) {
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
    config_data: web::Json<AuthProviderConfigRequest>,
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

    let provider_id = config_data.provider_id;
    
    // Verify the provider exists
    if let Err(e) = auth_provider_repo::get_provider_by_id(provider_id, &mut conn) {
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

    // Update each config item
    for config_item in &config_data.configs {
        // Skip empty secret values (these are placeholders for existing secrets)
        if config_item.is_secret && config_item.value.is_empty() {
            continue;
        }
        
        let new_config = NewAuthProviderConfig {
            auth_provider_id: provider_id,
            config_key: config_item.key.clone(),
            config_value: config_item.value.clone(),
            is_secret: config_item.is_secret,
        };
        
        if let Err(e) = auth_provider_repo::upsert_provider_config(new_config, &mut conn) {
            eprintln!("Error updating config for provider {}: {:?}", provider_id, e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to update provider configuration"
            }));
        }
    }

    // Enable the provider if it's not already enabled
    let provider = match auth_provider_repo::get_provider_by_id(provider_id, &mut conn) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error getting provider after config update: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to update provider configuration"
            }));
        }
    };
    
    if !provider.enabled {
        let update = AuthProviderUpdate {
            name: None,
            enabled: Some(true),
            is_default: None,
            updated_at: Some(Utc::now().naive_utc()),
        };
        
        if let Err(e) = auth_provider_repo::update_provider(provider_id, update, &mut conn) {
            eprintln!("Error enabling provider after config update: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to enable provider after configuration update"
            }));
        }
    }

    // Return the updated provider with configs
    match auth_provider_repo::get_provider_with_configs(provider_id, &mut conn) {
        Ok(provider) => HttpResponse::Ok().json(provider),
        Err(e) => {
            eprintln!("Error getting provider with configs after update: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Provider configuration updated, but failed to retrieve updated data"
            }))
        }
    }
}

// Generate OAuth authorization URL
pub async fn oauth_authorize(
    db_pool: web::Data<Pool>,
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
        // Get the provider configuration
        let client_id = match auth_provider_repo::get_provider_config(provider.id, "client_id", &mut conn) {
            Ok(config) => config.config_value,
            Err(e) => {
                eprintln!("Error getting client_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Microsoft authentication is not properly configured"
                }));
            }
        };
        
        let tenant_id = match auth_provider_repo::get_provider_config(provider.id, "tenant_id", &mut conn) {
            Ok(config) => config.config_value,
            Err(e) => {
                eprintln!("Error getting tenant_id for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Microsoft authentication is not properly configured"
                }));
            }
        };
        
        let redirect_uri = match auth_provider_repo::get_provider_config(provider.id, "redirect_uri", &mut conn) {
            Ok(config) => config.config_value,
            Err(e) => {
                eprintln!("Error getting redirect_uri for Microsoft provider: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Microsoft authentication is not properly configured"
                }));
            }
        };
        
        // Generate a random state
        let state = format!("{:x}", rand::random::<u128>());
        
        // Create the authorization URL
        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=User.Read&state={}",
            tenant_id, client_id, redirect_uri, state
        );
        
        // Store the state for validation when the user returns
        // In a real implementation, this would be stored in a database or cache
        // For now, we'll just return it in the response
        
        // Generate expiration timestamp (10 minutes from now)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let exp = now + (10 * 60); // 10 minutes
        
        let oauth_state = OAuthState {
            state: state.clone(),
            redirect_uri: oauth_request.redirect_uri.clone().unwrap_or_else(|| "/".to_string()),
            provider_type: provider.provider_type.clone(),
            exp,
        };
        
        // In a real implementation, you'd store this state
        // state_store.insert(state.clone(), oauth_state);
        
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

// We'd also need to implement the OAuth callback handler, but that requires 
// more infrastructure for state management that would need to be integrated 
// into the codebase. This would be part of the complete implementation. 