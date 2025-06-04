use diesel::prelude::*;
use chrono::{NaiveDateTime, Utc};
use crate::db::DbConnection;
use crate::models::{SyncHistory, NewSyncHistory, SyncHistoryUpdate};
use crate::schema::sync_history;

/// Create a new sync history record
pub fn create_sync_history(
    conn: &mut DbConnection,
    new_sync: NewSyncHistory,
) -> QueryResult<SyncHistory> {
    diesel::insert_into(sync_history::table)
        .values(&new_sync)
        .get_result(conn)
}

/// Update an existing sync history record
pub fn update_sync_history(
    conn: &mut DbConnection,
    sync_id: i32,
    update: SyncHistoryUpdate,
) -> QueryResult<SyncHistory> {
    diesel::update(sync_history::table.find(sync_id))
        .set(&update)
        .get_result(conn)
}

/// Get sync history by ID
pub fn get_sync_history_by_id(
    conn: &mut DbConnection,
    sync_id: i32,
) -> QueryResult<SyncHistory> {
    sync_history::table
        .find(sync_id)
        .first(conn)
}

/// Get the most recent completed sync
pub fn get_last_completed_sync(
    conn: &mut DbConnection,
) -> QueryResult<SyncHistory> {
    sync_history::table
        .filter(
            sync_history::status.eq("completed")
                .or(sync_history::status.eq("error"))
                .or(sync_history::status.eq("cancelled"))
        )
        .order(sync_history::started_at.desc())
        .first(conn)
}

/// Get all sync history records, ordered by most recent first
pub fn get_all_sync_history(
    conn: &mut DbConnection,
    limit: Option<i64>,
) -> QueryResult<Vec<SyncHistory>> {
    let mut query = sync_history::table
        .order(sync_history::started_at.desc())
        .into_boxed();
    
    if let Some(limit_val) = limit {
        query = query.limit(limit_val);
    }
    
    query.load(conn)
}

/// Get active sync sessions (running or starting)
pub fn get_active_syncs(
    conn: &mut DbConnection,
) -> QueryResult<Vec<SyncHistory>> {
    sync_history::table
        .filter(
            sync_history::status.eq("running")
                .or(sync_history::status.eq("starting"))
        )
        .order(sync_history::started_at.desc())
        .load(conn)
}

/// Delete old sync history records (older than specified days)
pub fn cleanup_old_sync_history(
    conn: &mut DbConnection,
    days_to_keep: i32,
) -> QueryResult<usize> {
    let cutoff_date = Utc::now().naive_utc() - chrono::Duration::days(days_to_keep as i64);
    
    diesel::delete(
        sync_history::table.filter(sync_history::started_at.lt(cutoff_date))
    )
    .execute(conn)
}

/// Mark a sync as completed with final counts
pub fn complete_sync(
    conn: &mut DbConnection,
    sync_id: i32,
    records_processed: i32,
    records_created: i32,
    records_updated: i32,
    error_message: Option<String>,
) -> QueryResult<SyncHistory> {
    let now = Utc::now().naive_utc();
    let update = SyncHistoryUpdate {
        status: Some("completed".to_string()),
        error_message,
        records_processed: Some(records_processed),
        records_created: Some(records_created),
        records_updated: Some(records_updated),
        records_failed: Some(0),
        completed_at: Some(Some(now)),
    };
    
    update_sync_history(conn, sync_id, update)
}

/// Mark a sync as cancelled with final counts
pub fn cancel_sync(
    conn: &mut DbConnection,
    sync_id: i32,
    records_processed: i32,
    records_created: i32,
    records_updated: i32,
    error_message: &str,
) -> QueryResult<SyncHistory> {
    let now = Utc::now().naive_utc();
    let update = SyncHistoryUpdate {
        status: Some("cancelled".to_string()),
        error_message: Some(error_message.to_string()),
        records_processed: Some(records_processed),
        records_created: Some(records_created),
        records_updated: Some(records_updated),
        records_failed: Some(0),
        completed_at: Some(Some(now)),
    };
    
    update_sync_history(conn, sync_id, update)
}

/// Mark a sync as failed
pub fn fail_sync(
    conn: &mut DbConnection,
    sync_id: i32,
    error_message: &str,
) -> QueryResult<SyncHistory> {
    let now = Utc::now().naive_utc();
    let update = SyncHistoryUpdate {
        status: Some("error".to_string()),
        error_message: Some(error_message.to_string()),
        completed_at: Some(Some(now)),
        records_processed: None,
        records_created: None,
        records_updated: None,
        records_failed: None,
    };
    
    update_sync_history(conn, sync_id, update)
}

/// Delete a sync history record by ID
pub fn delete_sync_history(
    conn: &mut DbConnection,
    sync_id: i32,
) -> QueryResult<usize> {
    diesel::delete(sync_history::table.find(sync_id))
        .execute(conn)
} 