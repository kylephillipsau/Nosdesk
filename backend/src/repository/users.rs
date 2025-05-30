use diesel::prelude::*;
use diesel::result::Error;

use crate::db::DbConnection;
use crate::models::*;
use crate::schema::*;

// User repository functions
pub fn get_users(conn: &mut DbConnection) -> Result<Vec<User>, Error> {
    users::table
        .order_by(users::name.asc())
        .load::<User>(conn)
}

// Get paginated users with filtering and sorting
pub fn get_paginated_users(
    conn: &mut DbConnection,
    page: i64,
    page_size: i64,
    sort_field: Option<String>,
    sort_direction: Option<String>,
    search: Option<String>,
    role: Option<String>,
) -> Result<(Vec<User>, i64), Error> {
    // Build the main query
    let mut query = users::table.into_boxed();
    
    // Apply filters if provided
    if let Some(search_term) = search.clone() {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            query = query.filter(
                users::name.ilike(search_pattern.clone())
                    .or(users::email.ilike(search_pattern.clone()))
                    .or(users::role.ilike(search_pattern.clone()))
                    .or(users::id.eq_any(
                        search_term.parse::<i32>().ok().map(|id| vec![id]).unwrap_or_default()
                    ))
            );
        }
    }
    
    // Handle role filter
    if let Some(role_filter) = role.clone() {
        if role_filter != "all" {
            query = query.filter(users::role.eq(role_filter));
        }
    }
    
    // Build a separate count query with the same filters
    let mut count_query = users::table.into_boxed();
    
    // Apply the same filters to the count query
    if let Some(search_term) = search {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            count_query = count_query.filter(
                users::name.ilike(search_pattern.clone())
                    .or(users::email.ilike(search_pattern.clone()))
                    .or(users::role.ilike(search_pattern.clone()))
                    .or(users::id.eq_any(
                        search_term.parse::<i32>().ok().map(|id| vec![id]).unwrap_or_default()
                    ))
            );
        }
    }
    
    // Handle role filter for count query
    if let Some(role_filter) = role {
        if role_filter != "all" {
            count_query = count_query.filter(users::role.eq(role_filter));
        }
    }
    
    // Count total matching records (before pagination)
    let total: i64 = count_query.count().get_result(conn)?;
    
    // Apply sorting to the main query
    match (sort_field.as_deref(), sort_direction.as_deref()) {
        (Some("id"), Some("asc")) => query = query.order(users::id.asc()),
        (Some("id"), _) => query = query.order(users::id.desc()),
        (Some("name"), Some("asc")) => query = query.order(users::name.asc()),
        (Some("name"), _) => query = query.order(users::name.desc()),
        (Some("email"), Some("asc")) => query = query.order(users::email.asc()),
        (Some("email"), _) => query = query.order(users::email.desc()),
        (Some("role"), Some("asc")) => query = query.order(users::role.asc()),
        (Some("role"), _) => query = query.order(users::role.desc()),
        _ => query = query.order(users::name.asc()), // Default sort by name
    }
    
    // Apply pagination
    let offset = (page - 1) * page_size;
    query = query.offset(offset).limit(page_size);
    
    // Execute the query
    let results = query.load::<User>(conn)?;
    
    Ok((results, total))
}

pub fn get_user_by_id(id: i32, conn: &mut DbConnection) -> Result<User, Error> {
    users::table
        .find(id)
        .first::<User>(conn)
}

pub fn get_user_by_uuid(uuid: &str, conn: &mut DbConnection) -> Result<User, Error> {
    users::table
        .filter(users::uuid.eq(uuid))
        .first::<User>(conn)
}

pub fn get_user_by_email(email: &str, conn: &mut DbConnection) -> Result<User, Error> {
    users::table
        .filter(users::email.eq(email))
        .first::<User>(conn)
}

/// Get user by Microsoft UUID
pub fn get_user_by_microsoft_uuid(conn: &mut DbConnection, microsoft_uuid: &str) -> QueryResult<User> {
    users::table.filter(users::microsoft_uuid.eq(microsoft_uuid)).first(conn)
}

pub fn create_user(
    user: NewUser,
    conn: &mut DbConnection,
) -> Result<User, Error> {
    diesel::insert_into(users::table)
        .values(user)
        .get_result(conn)
}

pub fn update_user(
    id: i32,
    user: UserUpdate,
    conn: &mut DbConnection,
) -> Result<User, Error> {
    diesel::update(users::table.find(id))
        .set(user)
        .get_result(conn)
}

pub fn delete_user(id: i32, conn: &mut DbConnection) -> Result<usize, Error> {
    diesel::delete(users::table.find(id)).execute(conn)
}

// Batch get users by UUIDs
pub fn get_users_by_uuids(uuids: &[String], conn: &mut DbConnection) -> Result<Vec<User>, Error> {
    users::table
        .filter(users::uuid.eq_any(uuids))
        .order_by(users::name.asc())
        .load::<User>(conn)
} 