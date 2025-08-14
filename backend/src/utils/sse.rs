// Removed unused import: use serde_json::json;
use crate::handlers::sse::{SseState, TicketEvent};
use crate::utils::jwt::JwtUtils;
use actix_web::web;
use chrono::Utc;

/// SSE broadcasting utilities for real-time ticket updates
pub struct SseBroadcaster;

impl SseBroadcaster {
    /// Generic function to broadcast any SSE event
    async fn broadcast_event(state: &web::Data<SseState>, event: TicketEvent) {
        println!("SSE: Broadcasting event: {:?}", event);
        state.broadcast_event(event).await;
        println!("SSE: Event broadcasted successfully");
    }

    /// Generic function to create and broadcast events with common fields
    async fn broadcast_generic_event<F>(state: &web::Data<SseState>, event_creator: F)
    where
        F: FnOnce(chrono::DateTime<Utc>) -> TicketEvent,
    {
        let event = event_creator(Utc::now());
        Self::broadcast_event(state, event).await;
    }

    /// Broadcast a ticket field update to all connected clients
    pub async fn broadcast_ticket_updated(
        state: &web::Data<SseState>,
        ticket_id: i32,
        field: &str,
        value: serde_json::Value,
        updated_by: &str,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::TicketUpdated {
                ticket_id,
                field: field.to_string(),
                value,
                updated_by: updated_by.to_string(),
                timestamp,
            }
        }).await;
    }

    /// Broadcast a comment addition to all connected clients
    pub async fn broadcast_comment_added(
        state: &web::Data<SseState>,
        ticket_id: i32,
        comment: serde_json::Value,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::CommentAdded {
                ticket_id,
                comment,
                timestamp,
            }
        }).await;
    }

    /// Broadcast a comment deletion to all connected clients
    pub async fn broadcast_comment_deleted(
        state: &web::Data<SseState>,
        ticket_id: i32,
        comment_id: i32,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::CommentDeleted {
                ticket_id,
                comment_id,
                timestamp,
            }
        }).await;
    }

    /// Broadcast a device linking event to all connected clients
    pub async fn broadcast_device_linked(
        state: &web::Data<SseState>,
        ticket_id: i32,
        device_id: i32,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::DeviceLinked {
                ticket_id,
                device_id,
                timestamp,
            }
        }).await;
    }

    /// Broadcast a device unlinking event to all connected clients
    pub async fn broadcast_device_unlinked(
        state: &web::Data<SseState>,
        ticket_id: i32,
        device_id: i32,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::DeviceUnlinked {
                ticket_id,
                device_id,
                timestamp,
            }
        }).await;
    }

    /// Broadcast a device update event to all connected clients
    pub async fn broadcast_device_updated(
        state: &web::Data<SseState>,
        device_id: i32,
        field: &str,
        value: serde_json::Value,
        updated_by: &str,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::DeviceUpdated {
                device_id,
                field: field.to_string(),
                value,
                updated_by: updated_by.to_string(),
                timestamp,
            }
        }).await;
    }

    /// Broadcast a ticket linking event to all connected clients
    pub async fn broadcast_ticket_linked(
        state: &web::Data<SseState>,
        ticket_id: i32,
        linked_ticket_id: i32,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::TicketLinked {
                ticket_id,
                linked_ticket_id,
                timestamp,
            }
        }).await;
    }

    /// Broadcast a ticket unlinking event to all connected clients
    pub async fn broadcast_ticket_unlinked(
        state: &web::Data<SseState>,
        ticket_id: i32,
        linked_ticket_id: i32,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::TicketUnlinked {
                ticket_id,
                linked_ticket_id,
                timestamp,
            }
        }).await;
    }

    /// Broadcast a project assignment event to all connected clients
    pub async fn broadcast_project_assigned(
        state: &web::Data<SseState>,
        ticket_id: i32,
        project_id: i32,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::ProjectAssigned {
                ticket_id,
                project_id,
                timestamp,
            }
        }).await;
    }

    /// Broadcast a project unassignment event to all connected clients
    pub async fn broadcast_project_unassigned(
        state: &web::Data<SseState>,
        ticket_id: i32,
        project_id: i32,
    ) {
        Self::broadcast_generic_event(state, |timestamp| {
            TicketEvent::ProjectUnassigned {
                ticket_id,
                project_id,
                timestamp,
            }
        }).await;
    }

    /// Get the number of currently connected SSE clients
    pub fn get_connected_clients_count(state: &web::Data<SseState>) -> usize {
        state.get_client_count()
    }

    /// Validate SSE authentication token from query parameters
    pub async fn validate_sse_token(
        token: Option<&String>,
        conn: &mut crate::db::DbConnection,
    ) -> Result<(crate::models::Claims, crate::models::User), crate::utils::jwt::JwtError> {
        let token_str = JwtUtils::extract_token_from_query(token)?;
        JwtUtils::validate_token_with_user_check(token_str, conn).await
    }
}

/// Macro for easy SSE broadcasting with error handling
#[macro_export]
macro_rules! broadcast_sse_event {
    ($state:expr, $method:ident, $($arg:expr),*) => {
        if let Some(state) = $state.as_ref() {
            crate::utils::sse::SseBroadcaster::$method(state, $($arg),*);
        }
    };
} 