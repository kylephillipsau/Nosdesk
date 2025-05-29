use diesel::prelude::*;
use crate::db::DbConnection;
use crate::models::{UserEmail, NewUserEmail, UserEmailUpdate};
use crate::schema::user_emails;
use chrono::Utc;

/// Create a new user email
pub fn create_user_email(
    conn: &mut DbConnection,
    new_email: NewUserEmail,
) -> Result<UserEmail, diesel::result::Error> {
    diesel::insert_into(user_emails::table)
        .values(&new_email)
        .get_result(conn)
}

/// Get all emails for a specific user
pub fn get_user_emails(
    conn: &mut DbConnection,
    user_id: i32,
) -> Result<Vec<UserEmail>, diesel::result::Error> {
    user_emails::table
        .filter(user_emails::user_id.eq(user_id))
        .order(user_emails::is_primary.desc())
        .then_order_by(user_emails::created_at.asc())
        .load::<UserEmail>(conn)
}

/// Get all emails for a specific user by UUID
pub fn get_user_emails_by_uuid(
    conn: &mut DbConnection,
    user_uuid: &str,
) -> Result<Vec<UserEmail>, diesel::result::Error> {
    use crate::schema::users;
    
    user_emails::table
        .inner_join(users::table.on(user_emails::user_id.eq(users::id)))
        .filter(users::uuid.eq(user_uuid))
        .select(user_emails::all_columns)
        .order(user_emails::is_primary.desc())
        .then_order_by(user_emails::created_at.asc())
        .load::<UserEmail>(conn)
}

/// Find a user by any of their email addresses
pub fn find_user_by_any_email(
    conn: &mut DbConnection,
    email: &str,
) -> Result<crate::models::User, diesel::result::Error> {
    use crate::schema::users;
    
    users::table
        .inner_join(user_emails::table.on(users::id.eq(user_emails::user_id)))
        .filter(user_emails::email.eq(email))
        .select(users::all_columns)
        .first::<crate::models::User>(conn)
}

/// Get the primary email for a user
pub fn get_primary_email(
    conn: &mut DbConnection,
    user_id: i32,
) -> Result<UserEmail, diesel::result::Error> {
    user_emails::table
        .filter(user_emails::user_id.eq(user_id))
        .filter(user_emails::is_primary.eq(true))
        .first::<UserEmail>(conn)
}

/// Check if an email address already exists in the system
pub fn email_exists(
    conn: &mut DbConnection,
    email: &str,
) -> Result<bool, diesel::result::Error> {
    use diesel::dsl::exists;
    use diesel::select;
    
    select(exists(
        user_emails::table.filter(user_emails::email.eq(email))
    )).get_result(conn)
}

/// Update a user email
pub fn update_user_email(
    conn: &mut DbConnection,
    email_id: i32,
    update: UserEmailUpdate,
) -> Result<UserEmail, diesel::result::Error> {
    let mut update_with_timestamp = update;
    update_with_timestamp.updated_at = Some(Utc::now().naive_utc());
    
    diesel::update(user_emails::table.find(email_id))
        .set(&update_with_timestamp)
        .get_result(conn)
}

/// Delete a user email (only if it's not the primary email)
pub fn delete_user_email(
    conn: &mut DbConnection,
    email_id: i32,
    user_id: i32,
) -> Result<usize, diesel::result::Error> {
    // First check if this is the primary email
    let email = user_emails::table
        .find(email_id)
        .filter(user_emails::user_id.eq(user_id))
        .first::<UserEmail>(conn)?;
    
    if email.is_primary {
        return Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::CheckViolation,
            Box::new("Cannot delete primary email".to_string())
        ));
    }
    
    diesel::delete(
        user_emails::table
            .find(email_id)
            .filter(user_emails::user_id.eq(user_id))
    ).execute(conn)
}

/// Set a new primary email for a user (unsets the old primary)
pub fn set_primary_email(
    conn: &mut DbConnection,
    user_id: i32,
    email_id: i32,
) -> Result<(), diesel::result::Error> {
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // First unset all primary emails for this user
        diesel::update(
            user_emails::table.filter(user_emails::user_id.eq(user_id))
        )
        .set((
            user_emails::is_primary.eq(false),
            user_emails::updated_at.eq(Utc::now().naive_utc())
        ))
        .execute(conn)?;
        
        // Then set the new primary email
        diesel::update(
            user_emails::table
                .find(email_id)
                .filter(user_emails::user_id.eq(user_id))
        )
        .set((
            user_emails::is_primary.eq(true),
            user_emails::updated_at.eq(Utc::now().naive_utc())
        ))
        .execute(conn)?;
        
        Ok(())
    })
}

/// Add multiple emails for a user (used during Microsoft Graph sync)
pub fn add_multiple_emails(
    conn: &mut DbConnection,
    user_id: i32,
    emails: Vec<(String, String, bool, String)>, // (email, type, verified, source)
) -> Result<Vec<UserEmail>, diesel::result::Error> {
    let new_emails: Vec<NewUserEmail> = emails
        .into_iter()
        .enumerate()
        .map(|(i, (email, email_type, verified, source))| NewUserEmail {
            user_id,
            email,
            email_type,
            is_primary: i == 0, // First email is primary
            verified,
            source: Some(source),
        })
        .collect();
    
    if new_emails.is_empty() {
        return Ok(Vec::new());
    }
    
    diesel::insert_into(user_emails::table)
        .values(&new_emails)
        .on_conflict(user_emails::email)
        .do_update()
        .set((
            user_emails::verified.eq(diesel::dsl::sql("EXCLUDED.verified")),
            user_emails::source.eq(diesel::dsl::sql("EXCLUDED.source")),
            user_emails::updated_at.eq(Utc::now().naive_utc())
        ))
        .get_results(conn)
}

/// Remove emails for a user that are no longer present in the source system
pub fn cleanup_obsolete_emails(
    conn: &mut DbConnection,
    user_id: i32,
    current_emails: &[String],
    source: &str,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        user_emails::table
            .filter(user_emails::user_id.eq(user_id))
            .filter(user_emails::source.eq(source))
            .filter(user_emails::email.ne_all(current_emails))
            .filter(user_emails::is_primary.eq(false)) // Never delete primary emails
    ).execute(conn)
}

/// Get all email addresses for a user as a simple list
pub fn get_user_email_addresses(
    conn: &mut DbConnection,
    user_id: i32,
) -> Result<Vec<String>, diesel::result::Error> {
    user_emails::table
        .filter(user_emails::user_id.eq(user_id))
        .select(user_emails::email)
        .load::<String>(conn)
}

/// Check if any of the provided emails belong to an existing user
pub fn find_user_by_any_of_emails(
    conn: &mut DbConnection,
    emails: &[String],
) -> Result<Option<crate::models::User>, diesel::result::Error> {
    use crate::schema::users;
    
    if emails.is_empty() {
        return Ok(None);
    }
    
    let result = users::table
        .inner_join(user_emails::table.on(users::id.eq(user_emails::user_id)))
        .filter(user_emails::email.eq_any(emails))
        .select(users::all_columns)
        .first::<crate::models::User>(conn)
        .optional()?;
    
    Ok(result)
} 