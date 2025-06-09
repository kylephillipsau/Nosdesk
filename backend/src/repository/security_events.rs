use diesel::prelude::*;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};
use anyhow::{Context, Result};

use crate::db::DbConnection;
use crate::models::{SecurityEvent, NewSecurityEvent, SecurityEventType, SecurityEventSeverity};
use crate::schema::security_events;

/// Create a new security event
pub fn create_event(
    conn: &mut DbConnection,
    new_event: NewSecurityEvent,
) -> Result<SecurityEvent> {
    diesel::insert_into(security_events::table)
        .values(&new_event)
        .get_result(conn)
        .context("Failed to create security event")
}

/// Helper function to create a security event with type safety
pub fn log_security_event(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
    event_type: SecurityEventType,
    severity: SecurityEventSeverity,
    ip_address: Option<ipnetwork::IpNetwork>,
    user_agent: Option<String>,
    location: Option<String>,
    details: Option<serde_json::Value>,
    session_id: Option<i32>,
) -> Result<SecurityEvent> {
    let new_event = NewSecurityEvent {
        user_uuid: *user_uuid,
        event_type: event_type.as_str().to_string(),
        ip_address: ip_address.map(|ip| ip.to_string()), // Convert to string
        user_agent,
        location,
        details,
        severity: severity.as_str().to_string(),
        session_id,
    };
    
    create_event(conn, new_event)
        .with_context(|| format!("Failed to log {} event for user {}", event_type, user_uuid))
}

/// Get security events for a user
pub fn get_user_events(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
    limit: Option<i64>,
) -> Result<Vec<SecurityEvent>> {
    let mut query = security_events::table
        .filter(security_events::user_uuid.eq(user_uuid))
        .order_by(security_events::created_at.desc())
        .into_boxed();
    
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    
    query.load::<SecurityEvent>(conn)
        .with_context(|| format!("Failed to get security events for user {}", user_uuid))
}

/// Get security events by type
pub fn get_events_by_type(
    conn: &mut DbConnection,
    event_type: SecurityEventType,
    limit: Option<i64>,
) -> Result<Vec<SecurityEvent>> {
    let mut query = security_events::table
        .filter(security_events::event_type.eq(event_type.as_str()))
        .order_by(security_events::created_at.desc())
        .into_boxed();
    
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    
    query.load::<SecurityEvent>(conn)
        .with_context(|| format!("Failed to get security events by type {}", event_type))
}

/// Get security events by severity
pub fn get_events_by_severity(
    conn: &mut DbConnection,
    severity: SecurityEventSeverity,
    limit: Option<i64>,
) -> Result<Vec<SecurityEvent>> {
    let mut query = security_events::table
        .filter(security_events::severity.eq(severity.as_str()))
        .order_by(security_events::created_at.desc())
        .into_boxed();
    
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    
    query.load::<SecurityEvent>(conn)
        .with_context(|| format!("Failed to get security events by severity {}", severity))
}

/// Get critical security events (for monitoring)
pub fn get_critical_events(
    conn: &mut DbConnection,
    since: NaiveDateTime,
) -> Result<Vec<SecurityEvent>> {
    security_events::table
        .filter(security_events::severity.eq("critical"))
        .filter(security_events::created_at.gt(since))
        .order_by(security_events::created_at.desc())
        .load::<SecurityEvent>(conn)
        .with_context(|| format!("Failed to get critical security events since {}", since))
}

/// Get recent failed login attempts for a user
pub fn get_recent_failed_logins(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
    since: NaiveDateTime,
) -> Result<Vec<SecurityEvent>> {
    security_events::table
        .filter(security_events::user_uuid.eq(user_uuid))
        .filter(security_events::event_type.eq("login_failed"))
        .filter(security_events::created_at.gt(since))
        .order_by(security_events::created_at.desc())
        .load::<SecurityEvent>(conn)
        .with_context(|| format!("Failed to get recent failed login attempts for user {} since {}", user_uuid, since))
}

/// Get recent MFA failures for a user (for rate limiting)
pub fn get_recent_mfa_failures(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
    since: NaiveDateTime,
) -> Result<Vec<SecurityEvent>> {
    security_events::table
        .filter(security_events::user_uuid.eq(user_uuid))
        .filter(security_events::event_type.eq("mfa_failed"))
        .filter(security_events::created_at.gt(since))
        .order_by(security_events::created_at.desc())
        .load::<SecurityEvent>(conn)
        .with_context(|| format!("Failed to get recent MFA failures for user {} since {}", user_uuid, since))
}

/// Count security events by type for a user in a time period
pub fn count_user_events_by_type(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
    event_type: SecurityEventType,
    since: NaiveDateTime,
) -> Result<i64> {
    security_events::table
        .filter(security_events::user_uuid.eq(user_uuid))
        .filter(security_events::event_type.eq(event_type.as_str()))
        .filter(security_events::created_at.gt(since))
        .count()
        .get_result(conn)
        .with_context(|| format!("Failed to count security events of type {} for user {} since {}", event_type, user_uuid, since))
}

/// Clean up old security events (for retention policy)
pub fn cleanup_old_events(
    conn: &mut DbConnection,
    older_than: NaiveDateTime,
) -> Result<usize> {
    diesel::delete(
        security_events::table.filter(security_events::created_at.lt(older_than))
    )
    .execute(conn)
    .with_context(|| format!("Failed to clean up old security events older than {}", older_than))
}

/// Get suspicious activity for monitoring
pub fn get_suspicious_activity(
    conn: &mut DbConnection,
    limit: Option<i64>,
) -> Result<Vec<SecurityEvent>> {
    let mut query = security_events::table
        .filter(
            security_events::event_type.eq("suspicious_activity")
            .or(security_events::severity.eq("critical"))
        )
        .order_by(security_events::created_at.desc())
        .into_boxed();
    
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    
    query.load::<SecurityEvent>(conn)
        .with_context(|| "Failed to get suspicious activity")
} 