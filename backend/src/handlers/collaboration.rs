use actix_web::{web, HttpResponse, Responder, Error, HttpRequest};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, ActorContext, Running, AsyncContext, Handler, Message, Addr};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use yrs::{Doc, Transact, ReadTxn, StateVector, Update};
use yrs::sync::{Awareness, Protocol, DefaultProtocol};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use bytes::Bytes;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

use crate::repository;
use crate::models::NewArticleContent;

// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
// Minimum time between saves for the same document
const MIN_SAVE_INTERVAL: Duration = Duration::from_secs(5);
// Maximum time a document can have pending changes before forcing a save
const MAX_PENDING_DURATION: Duration = Duration::from_secs(120);
// How long to wait before doing final save on empty room
const EMPTY_ROOM_FINAL_SAVE_DELAY: Duration = Duration::from_secs(2);
// How long to keep document state after room becomes empty
const EMPTY_ROOM_CLEANUP_DELAY: Duration = Duration::from_secs(300); // 5 minutes

// Simple handler to get article content by ticket ID
pub async fn get_article_content(
    pool: web::Data<crate::db::Pool>,
    doc_id: web::Path<String>,
) -> impl Responder {
    let doc_id = doc_id.into_inner();
    let clean_doc_id = doc_id.replace("/", "_");
    
    // Extract ticket ID from doc_id (format: "ticket-123")
    let ticket_id = match clean_doc_id.strip_prefix("ticket-").and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            println!("Invalid ticket ID format: {}", clean_doc_id);
            return HttpResponse::BadRequest().json("Invalid ticket ID format");
        }
    };
    
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };
    
    match repository::get_article_content_by_ticket_id(&mut conn, ticket_id) {
        Ok(article_content) => {
            println!("Retrieved article content for ticket {}", ticket_id);
            
            // Encode binary content as base64
            let content_base64 = general_purpose::STANDARD.encode(&article_content.content);
            
            HttpResponse::Ok().json(json!({
                "content": content_base64,
                "ticket_id": ticket_id
            }))
        },
        Err(e) => {
            println!("No article content found for ticket {}: {}", ticket_id, e);
            HttpResponse::Ok().json(json!({
                "content": "",
                "ticket_id": ticket_id
            }))
        }
    }
}

// ============= WebSocket implementation =============

// Document state tracking
#[derive(Clone)]
struct DocumentState {
    awareness: Arc<Awareness>,
    last_saved: Instant,
    has_pending_changes: bool,
    pending_since: Option<Instant>,
    sync_message_count: u32,
    room_empty_since: Option<Instant>, // Track when room became empty
    final_save_completed: bool, // Track if final save was done
}

impl DocumentState {
    fn new(awareness: Arc<Awareness>) -> Self {
        Self {
            awareness,
            last_saved: Instant::now(),
            has_pending_changes: false,
            pending_since: None,
            sync_message_count: 0,
            room_empty_since: None,
            final_save_completed: false,
        }
    }
    
    fn mark_changed(&mut self) {
        if !self.has_pending_changes {
            self.has_pending_changes = true;
            self.pending_since = Some(Instant::now());
        }
        self.sync_message_count += 1;
        
        // Reset room empty tracking since there's activity
        self.room_empty_since = None;
        self.final_save_completed = false;
    }
    
    fn mark_saved(&mut self) {
        self.last_saved = Instant::now();
        self.has_pending_changes = false;
        self.pending_since = None;
        self.sync_message_count = 0;
    }
    
    fn mark_room_empty(&mut self) {
        if self.room_empty_since.is_none() {
            self.room_empty_since = Some(Instant::now());
            self.final_save_completed = false;
        }
    }
    
    fn mark_room_active(&mut self) {
        self.room_empty_since = None;
        self.final_save_completed = false;
    }
    
    fn mark_final_save_completed(&mut self) {
        self.final_save_completed = true;
    }
    
    fn should_save(&self) -> bool {
        if !self.has_pending_changes {
            return false;
        }
        
        let now = Instant::now();
        
        // Save if enough time has passed since last save
        if now.duration_since(self.last_saved) >= MIN_SAVE_INTERVAL {
            return true;
        }
        
        // Force save if changes have been pending too long
        if let Some(pending_since) = self.pending_since {
            if now.duration_since(pending_since) >= MAX_PENDING_DURATION {
                return true;
            }
        }
        
        // Force save after 10 sync messages to prevent data loss
        if self.sync_message_count >= 10 {
            return true;
        }
        
        false
    }
    
    fn should_do_final_save(&self) -> bool {
        // Only do final save if room has been empty for a bit, changes exist, and we haven't done it yet
        if let Some(empty_since) = self.room_empty_since {
            let now = Instant::now();
            return !self.final_save_completed && 
                   (self.has_pending_changes || now.duration_since(empty_since) < Duration::from_secs(5)) &&
                   now.duration_since(empty_since) >= EMPTY_ROOM_FINAL_SAVE_DELAY;
        }
        false
    }
    
    fn should_cleanup(&self) -> bool {
        // Clean up document state after room has been empty for the cleanup delay and final save is done
        if let Some(empty_since) = self.room_empty_since {
            let now = Instant::now();
            return self.final_save_completed && 
                   now.duration_since(empty_since) >= EMPTY_ROOM_CLEANUP_DELAY;
        }
        false
    }
}

// Create app state to manage active documents and awareness
type DocumentId = String;
type SessionId = String;
type SessionInfo = (Addr<YjsWebSocket>, Instant); // (Socket address, last activity timestamp)
type RoomSessions = HashMap<DocumentId, HashMap<SessionId, SessionInfo>>;
type RoomSessionStore = Arc<Mutex<RoomSessions>>;
type DocumentStore = Arc<Mutex<HashMap<DocumentId, DocumentState>>>;

// Define shared app state for WebSocket connections
#[derive(Clone)]
pub struct YjsAppState {
    documents: DocumentStore,
    sessions: RoomSessionStore,
    pool: web::Data<crate::db::Pool>,
}

impl YjsAppState {
    pub fn new(pool: web::Data<crate::db::Pool>) -> Self {
        let state = YjsAppState {
            documents: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pool,
        };
        // Start the periodic cleanup and save task
        let state_clone = state.clone();
        actix::spawn(async move {
            use actix::clock::interval;
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds (was 10)
            loop {
                interval.tick().await;
                state_clone.cleanup_stale_sessions();
                state_clone.save_all_active_documents();
            }
        });
        state
    }

    // Save all active documents
    fn save_all_active_documents(&self) {
        let mut documents = self.documents.lock().unwrap();
        let mut saved_count = 0;
        let mut final_saved_count = 0;
        let mut cleaned_up_count = 0;
        let mut docs_to_remove = Vec::new();
        
        for (doc_id, doc_state) in documents.iter_mut() {
            // Regular saves for active documents
            if doc_state.should_save() {
                println!("Saving document {} with pending changes", doc_id);
                self.save_document_internal(doc_id, &doc_state.awareness);
                doc_state.mark_saved();
                saved_count += 1;
            }
            
            // Final save for empty rooms
            if doc_state.should_do_final_save() {
                println!("Performing final save for empty room: {}", doc_id);
                self.save_document_internal(doc_id, &doc_state.awareness);
                doc_state.mark_saved();
                doc_state.mark_final_save_completed();
                final_saved_count += 1;
            }
            
            // Clean up old empty documents
            if doc_state.should_cleanup() {
                println!("Cleaning up old document state: {}", doc_id);
                docs_to_remove.push(doc_id.clone());
                cleaned_up_count += 1;
            }
        }
        
        // Remove cleaned up documents
        for doc_id in docs_to_remove {
            documents.remove(&doc_id);
        }
        
        if saved_count > 0 || final_saved_count > 0 || cleaned_up_count > 0 {
            println!("Periodic maintenance completed: {} regular saves, {} final saves, {} cleanups", 
                    saved_count, final_saved_count, cleaned_up_count);
        }
    }

    // Get or create awareness for a document
    fn get_or_create_awareness(&self, doc_id: &str) -> Arc<Awareness> {
        let mut documents = self.documents.lock().unwrap();
        
        if let Some(doc_state) = documents.get(doc_id) {
            // Only log when document is accessed for the first time in a while, not on every message
            Arc::clone(&doc_state.awareness)
        } else {
            println!("Creating new awareness for document: {}", doc_id);
            let doc = Doc::new();
            let mut awareness = Awareness::new(doc);
            
            // Load existing content from database if available
            if let Some(ticket_id_str) = doc_id.strip_prefix("ticket-") {
                if let Ok(ticket_id) = ticket_id_str.parse::<i32>() {
                    match self.pool.get() {
                        Ok(mut conn) => {
                            match repository::get_article_content_by_ticket_id(&mut conn, ticket_id) {
                                Ok(article_content) => {
                                    if !article_content.content.is_empty() {
                                        println!("Loading existing content for ticket {} ({} bytes)", 
                                                ticket_id, article_content.content.len());
                                        
                                        // Apply the saved state to the document
                                        if let Ok(update) = Update::decode_v1(article_content.content.as_bytes()) {
                                            if let Err(e) = awareness.doc_mut().transact_mut().apply_update(update) {
                                                println!("Error applying saved state: {:?}", e);
                                            }
                                        } else {
                                            println!("Failed to decode saved state for ticket {}", ticket_id);
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("No existing content found for ticket {}: {:?}", ticket_id, e);
                                }
                            }
                        },
                        Err(e) => {
                            println!("Database connection error while loading document: {:?}", e);
                        }
                    }
                }
            }
            
            let awareness_arc = Arc::new(awareness);
            let doc_state = DocumentState::new(Arc::clone(&awareness_arc));
            documents.insert(doc_id.to_string(), doc_state);
            awareness_arc
        }
    }

    // Mark document as having pending changes
    fn mark_document_changed(&self, doc_id: &str) {
        if let Ok(mut documents) = self.documents.lock() {
            if let Some(doc_state) = documents.get_mut(doc_id) {
                doc_state.mark_changed();
            }
        }
    }

    // Register session
    fn register_session(&self, doc_id: &str, session_id: &str, addr: Addr<YjsWebSocket>) {
        let mut sessions = self.sessions.lock().unwrap();
        
        // Get or create the room for this document
        let room = sessions.entry(doc_id.to_string())
            .or_insert_with(HashMap::new);
        
        // Add this session to the room with current timestamp
        room.insert(session_id.to_string(), (addr, Instant::now()));
        
        // Mark document as having active sessions
        if let Ok(mut documents) = self.documents.lock() {
            if let Some(doc_state) = documents.get_mut(doc_id) {
                doc_state.mark_room_active();
            }
        }
        
        println!("Session {} joined document {} (room now has {} users)", session_id, doc_id, room.len());
    }

    // Update session activity timestamp
    fn update_session_activity(&self, doc_id: &str, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(room) = sessions.get_mut(doc_id) {
            if let Some(session_info) = room.get_mut(session_id) {
                // Update the timestamp
                session_info.1 = Instant::now();
            }
        }
    }
    
    // Remove session
    fn remove_session(&self, doc_id: &str, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(room) = sessions.get_mut(doc_id) {
            room.remove(session_id);
            println!("Session {} left document {} (room now has {} users)", session_id, doc_id, room.len());
            
            // If room is empty, mark it as empty but don't save immediately
            if room.is_empty() {
                println!("Room for document {} is now empty, will save after delay", doc_id);
                // Release the mutex to avoid deadlock
                drop(sessions);
                
                // Mark the document as having an empty room
                if let Ok(mut documents) = self.documents.lock() {
                    if let Some(doc_state) = documents.get_mut(doc_id) {
                        doc_state.mark_room_empty();
                    }
                }
            }
        }
    }
    
    // Clean up stale sessions
    fn cleanup_stale_sessions(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        let now = Instant::now();
        let mut stale_session_count = 0;
        let mut newly_empty_rooms = Vec::new();
        
        // First pass: collect stale sessions
        for (doc_id, room) in sessions.iter_mut() {
            let mut stale_sessions = Vec::new();
            let was_empty = room.is_empty();
            
            for (session_id, (_, last_active)) in room.iter() {
                if now.duration_since(*last_active) > CLIENT_TIMEOUT * 5 {
                    stale_sessions.push(session_id.clone());
                }
            }
            
            stale_session_count += stale_sessions.len();
            
            // Remove stale sessions from the room
            for session_id in stale_sessions.iter() {
                println!("Removing stale session {} from document {}", session_id, doc_id);
                room.remove(session_id);
            }
            
            // If room just became empty, mark it
            if !was_empty && room.is_empty() {
                newly_empty_rooms.push(doc_id.clone());
            }
        }
        
        // Log cleanup summary
        if stale_session_count > 0 {
            println!("Cleaned up {} stale sessions", stale_session_count);
        }
        
        // Release the sessions lock before updating document states
        drop(sessions);
        
        // Mark newly empty rooms
        if !newly_empty_rooms.is_empty() {
            if let Ok(mut documents) = self.documents.lock() {
                for doc_id in newly_empty_rooms {
                    if let Some(doc_state) = documents.get_mut(&doc_id) {
                        println!("Marking room {} as empty due to stale session cleanup", doc_id);
                        doc_state.mark_room_empty();
                    }
                }
            }
        }
    }

    // Force save a document immediately, ignoring timing constraints
    fn force_save_document(&self, doc_id: &str) {
        let documents = self.documents.lock().unwrap();
        if let Some(doc_state) = documents.get(doc_id) {
            println!("Force saving document {} on disconnect", doc_id);
            self.save_document_internal(doc_id, &doc_state.awareness);
            // Note: We don't mark as saved here because the lock is immutable
            // The next save cycle will update the saved timestamp
        }
    }

    // Helper method to save a document by ID
    fn save_document_by_id(&self, doc_id: &str) {
        let mut documents = self.documents.lock().unwrap();
        if let Some(doc_state) = documents.get_mut(doc_id) {
            // Force save regardless of timing constraints when explicitly called
            if doc_state.has_pending_changes {
                self.save_document_internal(doc_id, &doc_state.awareness);
                doc_state.mark_saved();
            }
        }
    }

    // Broadcast update to all sessions in a room except sender
    fn broadcast(&self, doc_id: &str, sender_id: &str, msg: &[u8]) {
        if msg.is_empty() {
            return;
        }
        
        let sessions = self.sessions.lock().unwrap();
        
        if let Some(room) = sessions.get(doc_id) {
            // Send message to all clients except the sender
            for (id, (addr, _)) in room {
                if id != sender_id {
                    addr.do_send(YjsMessage(Bytes::copy_from_slice(msg)));
                }
            }
        }
    }

    // Save document state to the database from awareness
    fn save_document_internal(&self, doc_id: &str, awareness: &Awareness) {
        // Check if it's a ticket document
        if let Some(ticket_id_str) = doc_id.strip_prefix("ticket-") {
            if let Ok(ticket_id) = ticket_id_str.parse::<i32>() {
                // Get binary content from the document
                let binary_content = {
                    let doc = awareness.doc();
                    let txn = doc.transact();
                    txn.encode_state_as_update_v1(&StateVector::default())
                };
                
                println!("Saving document content for ticket {} ({} bytes)", ticket_id, binary_content.len());
                
                // Save to database in a separate thread
                let pool = self.pool.clone();
                let content = binary_content.clone(); // Already Vec<u8>
                
                // Use actix to spawn a blocking operation
                actix::spawn(async move {
                    match pool.get() {
                        Ok(mut conn) => {
                            let new_content = NewArticleContent {
                                ticket_id,
                                content: general_purpose::STANDARD.encode(&content), // Convert binary to base64 string for storage
                            };
                            
                            match repository::update_article_content(&mut conn, ticket_id, new_content) {
                                Ok(_) => println!("Successfully saved document for ticket {}", ticket_id),
                                Err(e) => println!("Failed to save document for ticket {}: {:?}", ticket_id, e),
                            }
                        },
                        Err(e) => println!("Database connection error when saving document: {:?}", e),
                    }
                });
            }
        }
    }
}

// Message type for WebSocket communications
#[derive(Message)]
#[rtype(result = "()")]
struct YjsMessage(Bytes);

// WebSocket actor
struct YjsWebSocket {
    id: String,
    doc_id: String,
    app_state: YjsAppState,
    hb: Instant,
    protocol: DefaultProtocol,
}

impl YjsWebSocket {
    fn new(doc_id: String, app_state: YjsAppState) -> Self {
        let id = Uuid::new_v4().to_string();
        
        YjsWebSocket {
            id,
            doc_id,
            app_state,
            hb: Instant::now(),
            protocol: DefaultProtocol,
        }
    }
    
    // Handle heartbeat
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            let time_since_last_hb = Instant::now().duration_since(act.hb);
            
            if time_since_last_hb > CLIENT_TIMEOUT {
                println!("WebSocket Client heartbeat failed after {} seconds, disconnecting session {}", 
                        time_since_last_hb.as_secs(), act.id);
                act.app_state.remove_session(&act.doc_id, &act.id);
                ctx.stop();
                return;
            }
            
            // Only send ping if we haven't heard from client in a while
            if time_since_last_hb > HEARTBEAT_INTERVAL / 2 {
                ctx.ping(b"");
            }
        });
    }
    
    // Process incoming messages using the built-in protocol
    fn process_message(&mut self, msg: &[u8], ctx: &mut ws::WebsocketContext<Self>) {
        if msg.is_empty() {
            return;
        }
        
        self.app_state.update_session_activity(&self.doc_id, &self.id);
        
        // Get the awareness for this document
        let awareness = self.app_state.get_or_create_awareness(&self.doc_id);
        
        // Use the built-in protocol handler to process the message
        match self.protocol.handle(&awareness, msg) {
            Ok(messages) => {
                // Send any response messages back to the client
                for message in messages {
                    let encoded = message.encode_v1();
                    ctx.binary(encoded);
                }
                
                // Broadcast the entire message to other clients
                self.app_state.broadcast(&self.doc_id, &self.id, msg);
                
                // Mark document as changed after sync updates
                if msg.get(0) == Some(&0) { // MESSAGE_SYNC
                    self.app_state.mark_document_changed(&self.doc_id);
                }
            },
            Err(e) => {
                println!("Error handling protocol message: {:?}", e);
            }
        }
    }
}

impl Actor for YjsWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket connection started: {} for doc {}", self.id, self.doc_id);
        self.hb(ctx);
        self.app_state.register_session(&self.doc_id, &self.id, ctx.address());
        
        // The client will initiate the sync protocol by sending their state vector
        // We don't need to send anything on connection start
        println!("Waiting for client sync request for document {}", self.doc_id);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("WebSocket connection closed: {} for doc {}", self.id, self.doc_id);
        
        // Remove the session first
        self.app_state.remove_session(&self.doc_id, &self.id);
        
        // Only force save if this was the last session in the room
        // The periodic save task will handle regular saves
        let should_force_save = {
            let sessions = self.app_state.sessions.lock().unwrap();
            if let Some(room) = sessions.get(&self.doc_id) {
                room.is_empty() // Only force save if room is now empty
            } else {
                true // Room doesn't exist, so it was the last session
            }
        };
        
        if should_force_save {
            println!("Last session for document {}, performing final save", self.doc_id);
            self.app_state.force_save_document(&self.doc_id);
        }
        
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for YjsWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            },
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(ws::Message::Binary(bin)) => {
                self.hb = Instant::now();
                self.process_message(&bin, ctx);
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            },
            _ => (),
        }
    }
}

impl Handler<YjsMessage> for YjsWebSocket {
    type Result = ();

    fn handle(&mut self, msg: YjsMessage, ctx: &mut Self::Context) {
        ctx.binary(msg.0);
    }
}

// WebSocket connection handler - entry point for WebSocket requests
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<YjsAppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let doc_id = path.into_inner();
    println!("WebSocket connection request for document: {}", doc_id);
    
    // Extract and validate JWT token from query parameters
    let query_string = req.query_string();
    let token = query_string.split('&')
        .find(|param| param.starts_with("token="))
        .and_then(|param| param.split('=').nth(1))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No authentication token provided"))?;
    
    // Validate the token using existing auth logic
    if let Some(pool) = req.app_data::<web::Data<crate::db::Pool>>() {
        let mut conn = pool.get()
            .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;
        
        // Use JWT validation logic directly
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
        use crate::models::Claims;
        use crate::handlers::auth::JWT_SECRET;
        
        // Create validation with same requirements as auth handler
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 30;
        
        // Decode the token
        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &validation,
        ) {
            Ok(data) => data,
            Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid or expired token")),
        };
        
        // Verify user still exists in database
        let user_uuid = match crate::utils::parse_uuid(&token_data.claims.sub) {
            Ok(uuid) => uuid,
            Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid user UUID in token")),
        };

        match crate::repository::users::get_user_by_uuid(&user_uuid, &mut conn) {
            Ok(_) => (),
            Err(_) => return Err(actix_web::error::ErrorUnauthorized("User not found")),
        }
    } else {
        return Err(actix_web::error::ErrorInternalServerError("Database pool not available"));
    }
    
    println!("WebSocket authentication successful for document: {}", doc_id);
    let actor = YjsWebSocket::new(doc_id, app_state.get_ref().clone());
    ws::start(actor, &req, stream)
}

// Configure routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/article/{doc_id}", web::get().to(get_article_content))
            .route("/ws/{doc_id}", web::get().to(ws_handler))
    );
}