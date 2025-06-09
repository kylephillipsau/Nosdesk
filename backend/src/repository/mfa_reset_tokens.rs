use diesel::prelude::*;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc, Duration};

use crate::db::DbConnection;
use crate::models::{MfaResetToken, NewMfaResetToken, MfaResetTokenUpdate};
use crate::schema::mfa_reset_tokens;

/// Create a new MFA reset token
pub fn create_token(
    conn: &mut DbConnection,
    new_token: NewMfaResetToken,
) -> Result<MfaResetToken, diesel::result::Error> {
    diesel::insert_into(mfa_reset_tokens::table)
        .values(&new_token)
        .get_result(conn)
}

/// Helper function to create a token with default expiry (15 minutes)
pub fn create_reset_token(
    conn: &mut DbConnection,
    token: String,
    user_uuid: &Uuid,
    ip_address: Option<ipnetwork::IpNetwork>,
    user_agent: Option<String>,
    requires_admin_approval: bool,
) -> Result<MfaResetToken, diesel::result::Error> {
    let expires_at = Utc::now().naive_utc() + Duration::minutes(15);
    
    let new_token = NewMfaResetToken {
        token,
        user_uuid: *user_uuid,
        ip_address: ip_address.map(|ip| ip.to_string()), // Convert to string
        user_agent,
        expires_at,
        email_verified: false,
        admin_approved: !requires_admin_approval, // Auto-approve if not required
    };
    
    create_token(conn, new_token)
}

/// Get a token by its value
pub fn get_token(
    conn: &mut DbConnection,
    token: &str,
) -> Result<MfaResetToken, diesel::result::Error> {
    mfa_reset_tokens::table
        .find(token)
        .first::<MfaResetToken>(conn)
}

/// Get a valid (unused, non-expired) token
pub fn get_valid_token(
    conn: &mut DbConnection,
    token: &str,
) -> Result<MfaResetToken, diesel::result::Error> {
    mfa_reset_tokens::table
        .find(token)
        .filter(mfa_reset_tokens::is_used.eq(false))
        .filter(mfa_reset_tokens::expires_at.gt(Utc::now().naive_utc()))
        .first::<MfaResetToken>(conn)
}

/// Get pending tokens that require admin approval
pub fn get_pending_admin_approval(
    conn: &mut DbConnection,
) -> Result<Vec<MfaResetToken>, diesel::result::Error> {
    mfa_reset_tokens::table
        .filter(mfa_reset_tokens::admin_approved.eq(false))
        .filter(mfa_reset_tokens::is_used.eq(false))
        .filter(mfa_reset_tokens::expires_at.gt(Utc::now().naive_utc()))
        .order_by(mfa_reset_tokens::created_at.asc())
        .load::<MfaResetToken>(conn)
}

/// Mark token as used
pub fn mark_token_used(
    conn: &mut DbConnection,
    token: &str,
) -> Result<MfaResetToken, diesel::result::Error> {
    let update = MfaResetTokenUpdate {
        used_at: Some(Utc::now().naive_utc()),
        is_used: Some(true),
        email_verified: None,
        admin_approved: None,
        admin_approved_by: None,
        admin_approved_at: None,
    };
    
    diesel::update(mfa_reset_tokens::table.find(token))
        .set(&update)
        .get_result(conn)
}

/// Mark email as verified
pub fn mark_email_verified(
    conn: &mut DbConnection,
    token: &str,
) -> Result<MfaResetToken, diesel::result::Error> {
    let update = MfaResetTokenUpdate {
        used_at: None,
        is_used: None,
        email_verified: Some(true),
        admin_approved: None,
        admin_approved_by: None,
        admin_approved_at: None,
    };
    
    diesel::update(mfa_reset_tokens::table.find(token))
        .set(&update)
        .get_result(conn)
}

/// Approve token by admin
pub fn approve_token(
    conn: &mut DbConnection,
    token: &str,
    admin_uuid: &Uuid,
) -> Result<MfaResetToken, diesel::result::Error> {
    let update = MfaResetTokenUpdate {
        used_at: None,
        is_used: None,
        email_verified: None,
        admin_approved: Some(true),
        admin_approved_by: Some(*admin_uuid),
        admin_approved_at: Some(Utc::now().naive_utc()),
    };
    
    diesel::update(mfa_reset_tokens::table.find(token))
        .set(&update)
        .get_result(conn)
}

/// Check if token is ready for use (verified and approved if required)
pub fn is_token_ready(token: &MfaResetToken) -> bool {
    !token.is_used 
        && token.expires_at > Utc::now().naive_utc()
        && token.email_verified
        && token.admin_approved
}

/// Get active tokens for a user
pub fn get_user_active_tokens(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
) -> Result<Vec<MfaResetToken>, diesel::result::Error> {
    mfa_reset_tokens::table
        .filter(mfa_reset_tokens::user_uuid.eq(user_uuid))
        .filter(mfa_reset_tokens::is_used.eq(false))
        .filter(mfa_reset_tokens::expires_at.gt(Utc::now().naive_utc()))
        .order_by(mfa_reset_tokens::created_at.desc())
        .load::<MfaResetToken>(conn)
}

/// Revoke all active tokens for a user
pub fn revoke_user_tokens(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
) -> Result<usize, diesel::result::Error> {
    let update = MfaResetTokenUpdate {
        used_at: Some(Utc::now().naive_utc()),
        is_used: Some(true),
        email_verified: None,
        admin_approved: None,
        admin_approved_by: None,
        admin_approved_at: None,
    };
    
    diesel::update(
        mfa_reset_tokens::table
            .filter(mfa_reset_tokens::user_uuid.eq(user_uuid))
            .filter(mfa_reset_tokens::is_used.eq(false))
    )
    .set(&update)
    .execute(conn)
}

/// Clean up expired tokens
pub fn cleanup_expired_tokens(
    conn: &mut DbConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        mfa_reset_tokens::table.filter(mfa_reset_tokens::expires_at.lt(Utc::now().naive_utc()))
    )
    .execute(conn)
}

/// Count active tokens for a user in the last hour (rate limiting)
pub fn count_recent_tokens(
    conn: &mut DbConnection,
    user_uuid: &Uuid,
) -> Result<i64, diesel::result::Error> {
    let one_hour_ago = Utc::now().naive_utc() - Duration::hours(1);
    
    mfa_reset_tokens::table
        .filter(mfa_reset_tokens::user_uuid.eq(user_uuid))
        .filter(mfa_reset_tokens::created_at.gt(one_hour_ago))
        .count()
        .get_result(conn)
} 