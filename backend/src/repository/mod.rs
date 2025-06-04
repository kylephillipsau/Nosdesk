// Domain-specific modules
pub mod article_content;
pub mod auth_providers;
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

// Re-export all functions
pub use article_content::*;
pub use auth_providers::*;
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

// Note: We've completed the transition to a fully modular structure
// by removing the base.rs file and keeping only domain-specific modules.