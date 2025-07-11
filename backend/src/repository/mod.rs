// Domain-specific modules
pub mod article_content;
pub mod comments;
pub mod devices;
pub mod documentation;
pub mod linked_tickets;
pub mod projects;
pub mod sync_history;
pub mod tickets;
pub mod user_auth_identities;
pub mod user_emails;
pub mod users;

// Security and session management repositories
pub mod active_sessions;
pub mod security_events;
pub mod mfa_reset_tokens;

// Re-export all functions
pub use article_content::*;
pub use comments::*;
pub use devices::*;
pub use documentation::*;
pub use linked_tickets::*;
pub use projects::*;
pub use sync_history::*;
pub use tickets::*;
pub use user_auth_identities::*;
pub use user_emails::*;
pub use users::*;

// Security and session management
pub use active_sessions::*;
pub use security_events::*;
pub use mfa_reset_tokens::*;

// Note: We've completed the transition to a fully modular structure
// by removing the base.rs file and keeping only domain-specific modules.