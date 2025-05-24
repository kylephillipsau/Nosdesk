use std::env;

#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "Missing environment variable: {}", key),
        }
    }
}

// This allows ConfigError to be used with `?` in functions returning Result<_, Box<dyn std::error::Error>>
impl std::error::Error for ConfigError {}

// Helper to get an environment variable or return a ConfigError
fn get_env_var(name: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::Missing(name.to_string()))
}

pub fn get_microsoft_client_id() -> Result<String, ConfigError> {
    get_env_var("MICROSOFT_CLIENT_ID")
}

pub fn get_microsoft_tenant_id() -> Result<String, ConfigError> {
    get_env_var("MICROSOFT_TENANT_ID")
}

pub fn get_microsoft_client_secret() -> Result<String, ConfigError> {
    get_env_var("MICROSOFT_CLIENT_SECRET")
}

pub fn get_microsoft_redirect_uri() -> Result<String, ConfigError> {
    get_env_var("MICROSOFT_REDIRECT_URI")
} 