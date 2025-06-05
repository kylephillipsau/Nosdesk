use uuid::Uuid;
use crate::models::{NewUser, UserRole};

/// Builder for creating NewUser instances with sensible defaults
/// Follows the builder pattern for clean, readable user creation
pub struct NewUserBuilder {
    uuid: Uuid,
    name: String,
    email: String,
    role: UserRole,
    password_hash: Vec<u8>,
    pronouns: Option<String>,
    avatar_url: Option<String>,
    banner_url: Option<String>,
    avatar_thumb: Option<String>,
    microsoft_uuid: Option<Uuid>,
    // MFA fields with sensible defaults
    mfa_secret: Option<String>,
    mfa_enabled: Option<bool>,
    mfa_backup_codes: Option<serde_json::Value>,
}

impl NewUserBuilder {
    /// Create a new user builder with required fields
    pub fn new(name: String, email: String, role: UserRole) -> Self {
        Self {
            uuid: Uuid::new_v4(), // Generate UUID by default
            name,
            email,
            role,
            password_hash: Vec::new(), // Empty by default
            pronouns: None,
            avatar_url: None,
            banner_url: None,
            avatar_thumb: None,
            microsoft_uuid: None,
            // MFA defaults - disabled for new users
            mfa_secret: None,
            mfa_enabled: Some(false),
            mfa_backup_codes: None,
        }
    }

    /// Set a specific UUID (useful for testing or specific requirements)
    pub fn with_uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = uuid;
        self
    }

    /// Set password hash
    pub fn with_password_hash(mut self, hash: Vec<u8>) -> Self {
        self.password_hash = hash;
        self
    }

    /// Set pronouns
    pub fn with_pronouns(mut self, pronouns: Option<String>) -> Self {
        self.pronouns = pronouns;
        self
    }

    /// Set avatar information
    pub fn with_avatar(mut self, avatar_url: Option<String>, avatar_thumb: Option<String>) -> Self {
        self.avatar_url = avatar_url;
        self.avatar_thumb = avatar_thumb;
        self
    }

    /// Set banner URL
    pub fn with_banner(mut self, banner_url: Option<String>) -> Self {
        self.banner_url = banner_url;
        self
    }

    /// Set Microsoft UUID (for OAuth users)
    pub fn with_microsoft_uuid(mut self, microsoft_uuid: Option<Uuid>) -> Self {
        self.microsoft_uuid = microsoft_uuid;
        self
    }

    /// Build the final NewUser instance
    pub fn build(self) -> NewUser {
        NewUser {
            uuid: self.uuid,
            name: self.name,
            email: self.email,
            role: self.role,
            password_hash: self.password_hash,
            pronouns: self.pronouns,
            avatar_url: self.avatar_url,
            banner_url: self.banner_url,
            avatar_thumb: self.avatar_thumb,
            microsoft_uuid: self.microsoft_uuid,
            mfa_secret: self.mfa_secret,
            mfa_enabled: self.mfa_enabled.unwrap_or(false),
            mfa_backup_codes: self.mfa_backup_codes,
            passkey_credentials: None,
        }
    }
}

/// Convenience functions for common user creation patterns
impl NewUserBuilder {
    /// Create a local authentication user with password
    pub fn local_user(name: String, email: String, role: UserRole, password_hash: Vec<u8>) -> Self {
        Self::new(name, email, role)
            .with_password_hash(password_hash)
    }

    /// Create an OAuth user (no local password)
    pub fn oauth_user(name: String, email: String, role: UserRole) -> Self {
        Self::new(name, email, role)
    }

    /// Create a Microsoft OAuth user with Microsoft UUID
    pub fn microsoft_user(
        name: String, 
        email: String, 
        role: UserRole, 
        microsoft_uuid: Option<Uuid>
    ) -> Self {
        Self::new(name, email, role)
            .with_microsoft_uuid(microsoft_uuid)
    }

    /// Create an admin user with password
    pub fn admin_user(name: String, email: String, password_hash: Vec<u8>) -> Self {
        Self::new(name, email, UserRole::Admin)
            .with_password_hash(password_hash)
    }
}

/// Helper functions for email and name normalization
pub mod normalization {
    use crate::utils;

    /// Normalize user data for consistent storage
    pub fn normalize_user_data(name: &str, email: &str) -> (String, String) {
        (
            utils::normalize_string(name),
            utils::normalize_email(email),
        )
    }

    /// Normalize optional fields
    pub fn normalize_optional_string(value: Option<&String>) -> Option<String> {
        value.map(|s| utils::normalize_string(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user_builder_defaults() {
        let user = NewUserBuilder::new(
            "John Doe".to_string(),
            "john@example.com".to_string(),
            UserRole::User,
        ).build();

        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.role, UserRole::User);
        assert_eq!(user.mfa_enabled, false);
        assert!(user.mfa_secret.is_none());
        assert!(user.mfa_backup_codes.is_none());
    }

    #[test]
    fn test_local_user_convenience() {
        let password_hash = b"hashed_password".to_vec();
        let user = NewUserBuilder::local_user(
            "Jane Doe".to_string(),
            "jane@example.com".to_string(),
            UserRole::User,
            password_hash.clone(),
        ).build();

        assert_eq!(user.name, "Jane Doe");
        assert_eq!(user.password_hash, password_hash);
        assert_eq!(user.mfa_enabled, false);
    }

    #[test]
    fn test_oauth_user_convenience() {
        let user = NewUserBuilder::oauth_user(
            "OAuth User".to_string(),
            "oauth@example.com".to_string(),
            UserRole::User,
        ).build();

        assert_eq!(user.name, "OAuth User");
        assert!(user.password_hash.is_empty());
        assert_eq!(user.mfa_enabled, false);
    }

    #[test]
    fn test_microsoft_user_convenience() {
        let ms_uuid = Uuid::new_v4();
        let user = NewUserBuilder::microsoft_user(
            "MS User".to_string(),
            "ms@example.com".to_string(),
            UserRole::User,
            Some(ms_uuid),
        ).build();

        assert_eq!(user.name, "MS User");
        assert_eq!(user.microsoft_uuid, Some(ms_uuid));
        assert_eq!(user.mfa_enabled, false);
    }
} 