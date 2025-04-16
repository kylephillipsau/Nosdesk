use diesel::prelude::*;
use chrono::Utc;
use crate::db::DbConnection;
use crate::models::{
    AuthProvider, AuthProviderConfig, AuthProviderConfigResponse,
    AuthProviderUpdate, AuthProviderWithConfig, NewAuthProvider,
    NewAuthProviderConfig
};
use crate::schema::{auth_providers, auth_provider_configs};

pub fn get_all_providers(conn: &mut DbConnection) -> Result<Vec<AuthProvider>, diesel::result::Error> {
    auth_providers::table
        .order_by(auth_providers::id.asc())
        .load::<AuthProvider>(conn)
}

pub fn get_provider_by_id(id: i32, conn: &mut DbConnection) -> Result<AuthProvider, diesel::result::Error> {
    auth_providers::table
        .find(id)
        .first::<AuthProvider>(conn)
}

pub fn get_provider_by_type(provider_type: &str, conn: &mut DbConnection) -> Result<AuthProvider, diesel::result::Error> {
    auth_providers::table
        .filter(auth_providers::provider_type.eq(provider_type))
        .first::<AuthProvider>(conn)
}

pub fn get_default_provider(conn: &mut DbConnection) -> Result<AuthProvider, diesel::result::Error> {
    auth_providers::table
        .filter(auth_providers::is_default.eq(true))
        .first::<AuthProvider>(conn)
}

pub fn get_enabled_providers(conn: &mut DbConnection) -> Result<Vec<AuthProvider>, diesel::result::Error> {
    auth_providers::table
        .filter(auth_providers::enabled.eq(true))
        .order_by(auth_providers::is_default.desc())
        .load::<AuthProvider>(conn)
}

pub fn create_provider(provider: NewAuthProvider, conn: &mut DbConnection) -> Result<AuthProvider, diesel::result::Error> {
    diesel::insert_into(auth_providers::table)
        .values(provider)
        .get_result::<AuthProvider>(conn)
}

pub fn update_provider(id: i32, update: AuthProviderUpdate, conn: &mut DbConnection) -> Result<AuthProvider, diesel::result::Error> {
    let update_with_timestamp = AuthProviderUpdate {
        updated_at: Some(Utc::now().naive_utc()),
        ..update
    };
    
    diesel::update(auth_providers::table.find(id))
        .set(update_with_timestamp)
        .get_result::<AuthProvider>(conn)
}

pub fn set_default_provider(id: i32, conn: &mut DbConnection) -> Result<(), diesel::result::Error> {
    // Start a transaction
    conn.transaction(|conn| {
        // First, set all providers to not default
        diesel::update(auth_providers::table)
            .set(auth_providers::is_default.eq(false))
            .execute(conn)?;
        
        // Then, set the specified provider as default
        diesel::update(auth_providers::table.find(id))
            .set(auth_providers::is_default.eq(true))
            .execute(conn)?;
        
        Ok(())
    })
}

pub fn delete_provider(id: i32, conn: &mut DbConnection) -> Result<usize, diesel::result::Error> {
    diesel::delete(auth_providers::table.find(id))
        .execute(conn)
}

// Config functions
pub fn get_provider_configs(provider_id: i32, conn: &mut DbConnection) -> Result<Vec<AuthProviderConfig>, diesel::result::Error> {
    auth_provider_configs::table
        .filter(auth_provider_configs::auth_provider_id.eq(provider_id))
        .order_by(auth_provider_configs::config_key.asc())
        .load::<AuthProviderConfig>(conn)
}

pub fn get_provider_config(provider_id: i32, key: &str, conn: &mut DbConnection) -> Result<AuthProviderConfig, diesel::result::Error> {
    auth_provider_configs::table
        .filter(auth_provider_configs::auth_provider_id.eq(provider_id))
        .filter(auth_provider_configs::config_key.eq(key))
        .first::<AuthProviderConfig>(conn)
}

pub fn upsert_provider_config(config: NewAuthProviderConfig, conn: &mut DbConnection) -> Result<AuthProviderConfig, diesel::result::Error> {
    // Check if config exists
    let existing = auth_provider_configs::table
        .filter(auth_provider_configs::auth_provider_id.eq(config.auth_provider_id))
        .filter(auth_provider_configs::config_key.eq(&config.config_key))
        .first::<AuthProviderConfig>(conn)
        .optional()?;
    
    if let Some(existing_config) = existing {
        // Update existing config
        diesel::update(auth_provider_configs::table.find(existing_config.id))
            .set((
                auth_provider_configs::config_value.eq(&config.config_value),
                auth_provider_configs::is_secret.eq(config.is_secret),
                auth_provider_configs::updated_at.eq(Utc::now().naive_utc())
            ))
            .get_result::<AuthProviderConfig>(conn)
    } else {
        // Insert new config
        diesel::insert_into(auth_provider_configs::table)
            .values(config)
            .get_result::<AuthProviderConfig>(conn)
    }
}

pub fn delete_provider_config(provider_id: i32, key: &str, conn: &mut DbConnection) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        auth_provider_configs::table
            .filter(auth_provider_configs::auth_provider_id.eq(provider_id))
            .filter(auth_provider_configs::config_key.eq(key))
    )
    .execute(conn)
}

// Combined queries
pub fn get_provider_with_configs(provider_id: i32, conn: &mut DbConnection) -> Result<AuthProviderWithConfig, diesel::result::Error> {
    let provider = get_provider_by_id(provider_id, conn)?;
    let configs = get_provider_configs(provider_id, conn)?;
    
    let config_responses: Vec<AuthProviderConfigResponse> = configs.into_iter()
        .map(|config| {
            let value = if config.is_secret {
                // Don't return actual secret values, just indicate they exist
                String::from("******")
            } else {
                config.config_value
            };
            
            AuthProviderConfigResponse {
                key: config.config_key,
                value,
                is_secret: config.is_secret,
            }
        })
        .collect();
    
    Ok(AuthProviderWithConfig {
        id: provider.id,
        provider_type: provider.provider_type,
        name: provider.name,
        enabled: provider.enabled,
        is_default: provider.is_default,
        configs: config_responses,
    })
}

pub fn get_all_providers_with_configs(conn: &mut DbConnection) -> Result<Vec<AuthProviderWithConfig>, diesel::result::Error> {
    let providers = get_all_providers(conn)?;
    
    let mut result = Vec::new();
    for provider in providers {
        let provider_id = provider.id;
        let configs = get_provider_configs(provider_id, conn)?;
        
        let config_responses: Vec<AuthProviderConfigResponse> = configs.into_iter()
            .map(|config| {
                let value = if config.is_secret {
                    // Don't return actual secret values, just indicate they exist
                    String::from("******")
                } else {
                    config.config_value
                };
                
                AuthProviderConfigResponse {
                    key: config.config_key,
                    value,
                    is_secret: config.is_secret,
                }
            })
            .collect();
        
        result.push(AuthProviderWithConfig {
            id: provider.id,
            provider_type: provider.provider_type,
            name: provider.name,
            enabled: provider.enabled,
            is_default: provider.is_default,
            configs: config_responses,
        });
    }
    
    Ok(result)
} 