use actix_web::{web, HttpResponse, Responder, Error, HttpRequest};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, ActorContext, Running, AsyncContext, Handler, Message, Addr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::io::{Cursor, Write};
use yrs::{Doc, Transact, ReadTxn, StateVector, Update};
use yrs::sync::{Awareness, AwarenessUpdate};
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};
use yrs::GetString;
use bytes::Bytes;
use uuid::Uuid;

use crate::repository;
use crate::models::NewArticleContent;

// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
// How often to check and clean up stale sessions
const CLEANUP_INTERVAL: Duration = Duration::from_secs(30);

// Y.js protocol message types
const MESSAGE_SYNC: u8 = 0;
const MESSAGE_AWARENESS: u8 = 1;
const MESSAGE_AUTH: u8 = 2;
const MESSAGE_QUERY_AWARENESS: u8 = 3;

// Original REST handlers for syncing article content

#[derive(Debug, Serialize, Deserialize)]
pub struct CollaborativeUpdate {
    pub doc_id: String,
    pub content: String,
}

// Simple handler to sync ticket article content
pub async fn sync_ticket_article(
    pool: web::Data<crate::db::Pool>,
    update: web::Json<CollaborativeUpdate>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    let doc_id = update.doc_id.clone();
    println!("Syncing content for document: {}", doc_id);
    
    let ticket_id = match doc_id.strip_prefix("ticket-").and_then(|id| id.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            println!("Invalid ticket ID format: {}", doc_id);
            return HttpResponse::BadRequest().json("Invalid ticket ID format");
        }
    };
    
    // Temporarily store content as is, assuming it's binary data
    // TODO: Implement base64 decoding once dependency is added
    let binary_content = update.content.as_bytes().to_vec();
    // let binary_content = match base64::decode(&update.content) {
    //     Ok(data) => data,
    //     Err(e) => {
    //         println!("Failed to decode base64 content: {}", e);
    //         return HttpResponse::BadRequest().json("Invalid content format");
    //     }
    // };

    let new_article_content = NewArticleContent {
        content: binary_content,
        ticket_id,
    };

    match repository::update_article_content(&mut conn, ticket_id, new_article_content) {
        Ok(article) => {
            println!("Successfully saved article for ticket {}", ticket_id);
            HttpResponse::Ok().json(json!({ 
                "status": "success", 
                "message": "Article synchronized", 
                "article_id": article.id 
            }))
        },
        Err(e) => {
            println!("Failed to sync article for ticket {}: {:?}", ticket_id, e);
            HttpResponse::InternalServerError().json(format!("Failed to sync article: {}", e))
        }
    }
}

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
            
            // Temporarily return content as is, assuming it's binary data
            // TODO: Implement base64 encoding once dependency is added
            let content_string = String::from_utf8_lossy(&article_content.content).to_string();
            // let content_base64 = base64::encode(&article_content.content);
            
            HttpResponse::Ok().json(json!({
                "content": content_string,
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

// Create app state to manage active documents and awareness
type DocumentId = String;
type DocumentStore = Arc<Mutex<HashMap<DocumentId, Doc>>>;
type SessionId = String;
type SessionInfo = (Addr<YjsWebSocket>, Instant); // (Socket address, last activity timestamp)
type RoomSessions = HashMap<DocumentId, HashMap<SessionId, SessionInfo>>;
type RoomSessionStore = Arc<Mutex<RoomSessions>>;

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
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
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
        let sessions = self.sessions.lock().unwrap();
        let active_docs: Vec<String> = sessions.keys().cloned().collect();
        drop(sessions); // Release the lock before saving
        for doc_id in active_docs {
            self.save_document(&doc_id);
        }
        println!("Periodic save completed for all active documents");
    }

    // Get or create document 
    fn get_or_create_doc(&self, doc_id: &str) -> Doc {
        let mut docs = self.documents.lock().unwrap();
        
        if !docs.contains_key(doc_id) {
            println!("Creating new document: {}", doc_id);
            let doc = Doc::new();
            docs.insert(doc_id.to_string(), doc.clone());
            doc
        } else {
            println!("Using existing document: {}", doc_id);
            docs.get(doc_id).unwrap().clone()
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
        
        println!("Session {} joined document {}", session_id, doc_id);
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
            println!("Session {} left document {}", session_id, doc_id);
            
            // If room is empty, consider saving the document
            if room.is_empty() {
                println!("Room for document {} is empty, saving state", doc_id);
                // Release the mutex to avoid deadlock when saving
                drop(sessions);
                self.save_document(doc_id);
                println!("Completed saving state for document {} after room emptied", doc_id);
                // Re-acquire the mutex
                sessions = self.sessions.lock().unwrap();
            }
        }
    }
    
    // Clean up stale sessions
    fn cleanup_stale_sessions(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        let now = Instant::now();
        let mut docs_to_clean = Vec::new();
        let mut stale_session_count = 0;
        
        // First pass: collect stale sessions
        for (doc_id, room) in sessions.iter_mut() {
            let mut stale_sessions = Vec::new();
            
            for (session_id, (_, last_active)) in room.iter() {
                if now.duration_since(*last_active) > CLIENT_TIMEOUT * 3 {
                    stale_sessions.push(session_id.clone());
                }
            }
            
            stale_session_count += stale_sessions.len();
            
            // Remove stale sessions from the room
            for session_id in stale_sessions.iter() {
                println!("Removing stale session {} from document {}", session_id, doc_id);
                room.remove(session_id);
            }
            
            // Mark document for potential cleaning if room is empty
            if room.is_empty() {
                docs_to_clean.push(doc_id.clone());
            }
        }
        
        // Log cleanup summary
        if stale_session_count > 0 {
            println!("Cleaned up {} stale sessions", stale_session_count);
        }
        
        // Process empty rooms and save their state
        for doc_id in docs_to_clean {
            println!("Room for document {} is empty, saving state", &doc_id);
            // Release the mutex before calling save_document
            drop(sessions);
            self.save_document(&doc_id);
            // Re-acquire the mutex
            sessions = self.sessions.lock().unwrap();
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

    // Save document state to the database
    fn save_document(&self, doc_id: &str) {
        let docs = self.documents.lock().unwrap();
        
        if let Some(doc) = docs.get(doc_id) {
            // Check if it's a ticket document
            if let Some(ticket_id_str) = doc_id.strip_prefix("ticket-") {
                if let Ok(ticket_id) = ticket_id_str.parse::<i32>() {
                    // Get binary content from the document
                    let binary_content = {
                        let txn = doc.transact();
                        txn.encode_state_as_update_v1(&StateVector::default())
                    };
                    
                    println!("Saving document content for ticket {}", ticket_id);
                    
                    // Save to database in a separate thread
                    let pool = self.pool.clone();
                    let content = binary_content.clone(); // Already Vec<u8>
                    
                    // Use actix to spawn a blocking operation
                    actix::spawn(async move {
                        match pool.get() {
                            Ok(mut conn) => {
                                let new_content = NewArticleContent {
                                    ticket_id,
                                    content,
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
}

impl YjsWebSocket {
    fn new(doc_id: String, app_state: YjsAppState) -> Self {
        let id = Uuid::new_v4().to_string();
        
        YjsWebSocket {
            id,
            doc_id,
            app_state,
            hb: Instant::now(),
        }
    }
    
    // Handle heartbeat
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("WebSocket Client heartbeat failed, disconnecting!");
                act.app_state.remove_session(&act.doc_id, &act.id);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
    
    // Process incoming messages according to Y-protocol
    fn process_message(&mut self, msg: &[u8], ctx: &mut ws::WebsocketContext<Self>) {
        if msg.is_empty() {
            return;
        }
        
        self.app_state.update_session_activity(&self.doc_id, &self.id);
        
        // First byte indicates the message type in Y-protocol
        match msg[0] {
            // Sync protocol message
            MESSAGE_SYNC => {
                self.app_state.broadcast(&self.doc_id, &self.id, msg);
                // After processing a sync message, trigger a save to ensure the backend captures the latest state
                self.app_state.save_document(&self.doc_id);
            },
            
            // Awareness protocol message
            MESSAGE_AWARENESS => {
                self.app_state.broadcast(&self.doc_id, &self.id, msg);
            },
            
            // Query awareness (used when client wants to get current awareness state)
            MESSAGE_QUERY_AWARENESS => {
                self.app_state.broadcast(&self.doc_id, &self.id, msg);
            },
            
            // Unknown message type - ignore
            _ => {
                println!("Received unknown message type: {}", msg[0]);
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
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("WebSocket connection closed: {} for doc {}", self.id, self.doc_id);
        self.app_state.remove_session(&self.doc_id, &self.id);
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
    
    let actor = YjsWebSocket::new(doc_id, app_state.get_ref().clone());
    ws::start(actor, &req, stream)
}

// Configure routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/sync", web::post().to(sync_ticket_article))
            .route("/article/{doc_id}", web::get().to(get_article_content))
            .route("/ws/{doc_id}", web::get().to(ws_handler))
    );
}