use diesel::prelude::*;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

use crate::db::DbConnection;
use crate::models::{ActiveSession, NewActiveSession, ActiveSessionUpdate};
use crate::schema::active_sessions;

/// Create a new active session
pub fn create_session(
    conn: &mut DbConnection,
    new_session: NewActiveSession,
) -> Result<ActiveSession, diesel::result::Error> {
    diesel::insert_into(active_sessions::table)
        .values(&new_session)
        .get_result(conn)
}

/// Get all active sessions for a user
pub fn get_user_sessions(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
) -> Result<Vec<ActiveSession>, diesel::result::Error> {
    active_sessions::table
        .filter(active_sessions::user_uuid.eq(user_uuid))
        .filter(active_sessions::expires_at.gt(Utc::now().naive_utc()))
        .order_by(active_sessions::last_active.desc())
        .load::<ActiveSession>(conn)
}

/// Get a session by token hash
pub fn get_session_by_token(
    conn: &mut DbConnection,
    token: &str,
) -> Result<ActiveSession, diesel::result::Error> {
    active_sessions::table
        .filter(active_sessions::session_token.eq(token))
        .first::<ActiveSession>(conn)
}

/// Update session last active time
pub fn update_session_activity(
    conn: &mut DbConnection,
    session_id: i32,
) -> Result<ActiveSession, diesel::result::Error> {
    let update = ActiveSessionUpdate {
        last_active: Some(Utc::now().naive_utc()),
        expires_at: None,
        is_current: None,
    };
    
    diesel::update(active_sessions::table.find(session_id))
        .set(&update)
        .get_result(conn)
}

/// Mark a session as current and all others for the user as not current
pub fn set_current_session(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
    session_id: i32,
) -> Result<(), diesel::result::Error> {
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // First, mark all user sessions as not current
        diesel::update(active_sessions::table.filter(active_sessions::user_uuid.eq(user_uuid)))
            .set(active_sessions::is_current.eq(false))
            .execute(conn)?;
        
        // Then mark the specific session as current
        diesel::update(active_sessions::table.find(session_id))
            .set(active_sessions::is_current.eq(true))
            .execute(conn)?;
        
        Ok(())
    })
}

/// Revoke a specific session
pub fn revoke_session(
    conn: &mut DbConnection,
    session_id: i32,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(active_sessions::table.find(session_id))
        .execute(conn)
}

/// Revoke all sessions for a user except the current one
pub fn revoke_other_sessions(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
    current_session_id: Option<i32>,
) -> Result<usize, diesel::result::Error> {
    match current_session_id {
        Some(session_id) => {
            diesel::delete(
                active_sessions::table
                    .filter(active_sessions::user_uuid.eq(user_uuid))
                    .filter(active_sessions::id.ne(session_id))
            )
            .execute(conn)
        }
        None => {
            diesel::delete(
                active_sessions::table.filter(active_sessions::user_uuid.eq(user_uuid))
            )
            .execute(conn)
        }
    }
}

/// Revoke all sessions for a user
pub fn revoke_all_sessions(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        active_sessions::table.filter(active_sessions::user_uuid.eq(user_uuid))
    )
    .execute(conn)
}

/// Clean up expired sessions
pub fn cleanup_expired_sessions(
    conn: &mut DbConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        active_sessions::table.filter(active_sessions::expires_at.lt(Utc::now().naive_utc()))
    )
    .execute(conn)
}

/// Count active sessions for a user
pub fn count_user_sessions(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
) -> Result<i64, diesel::result::Error> {
    active_sessions::table
        .filter(active_sessions::user_uuid.eq(user_uuid))
        .filter(active_sessions::expires_at.gt(Utc::now().naive_utc()))
        .count()
        .get_result(conn)
} 