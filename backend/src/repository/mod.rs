// Domain-specific modules
pub mod auth_providers;
pub mod documentation;
pub mod tickets;
pub mod devices;
pub mod comments;
pub mod users;
pub mod projects;
pub mod linked_tickets;
pub mod document_updates;
pub mod article_content;
pub mod user_auth_identities;

// Re-export from domain-specific modules
pub use documentation::*;
pub use tickets::*;
pub use devices::*;
pub use comments::*;
pub use users::*;
pub use projects::*;
pub use linked_tickets::*;
pub use document_updates::*;
pub use article_content::*;
pub use auth_providers::*;

// Note: We've completed the transition to a fully modular structure
// by removing the base.rs file and keeping only domain-specific modules.