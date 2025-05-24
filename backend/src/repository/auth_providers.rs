use diesel::prelude::*;
use chrono::Utc;
use crate::db::DbConnection;
use crate::models::{
    AuthProvider, AuthProviderUpdate, NewAuthProvider
};
use crate::schema::auth_providers;

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

pub fn get_default_provider_by_type(provider_type: &str, conn: &mut DbConnection) -> Result<AuthProvider, diesel::result::Error> {
    auth_providers::table
        .filter(auth_providers::provider_type.eq(provider_type))
        .filter(auth_providers::enabled.eq(true))
        .order_by(auth_providers::is_default.desc())
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