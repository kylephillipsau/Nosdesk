use diesel::prelude::*;
use uuid::Uuid;
use chrono::Utc;

use crate::db::DbConnection;
use crate::models::{ActiveSession, NewActiveSession};
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

 