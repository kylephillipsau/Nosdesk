use anyhow::{Result, anyhow};
use base32;
use bcrypt::{verify as bcrypt_verify, hash as bcrypt_hash, DEFAULT_COST};
use qrcode::{QrCode, render::svg};
use base64::{Engine as _, engine::general_purpose};
use totp_rs::{Algorithm as TotpAlgorithm, TOTP, Secret};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng, RngCore};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use zeroize::ZeroizeOnDrop;
use uuid::Uuid;
use serde_json::Value;

use crate::models::{User, UserRole};
use crate::db::DbConnection;
use crate::repository;

/// Secure wrapper for sensitive strings that zeros memory on drop
#[derive(ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: String) -> Self {
        Self(s)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// MFA verification result
#[derive(Debug, Clone)]
pub struct MfaVerificationResult {
    pub is_valid: bool,
    pub backup_code_used: Option<String>,
    pub requires_backup_code_regeneration: bool,
}

/// Get encryption key from environment (must be 32 bytes for AES-256-GCM)
fn get_encryption_key() -> Result<[u8; 32]> {
    let key_hex = std::env::var("MFA_ENCRYPTION_KEY")
        .map_err(|_| anyhow!("MFA_ENCRYPTION_KEY environment variable not set"))?;
    
    if key_hex.len() != 64 {
        return Err(anyhow!("MFA_ENCRYPTION_KEY must be exactly 64 hex characters (32 bytes)"));
    }
    
    let mut key = [0u8; 32];
    hex::decode_to_slice(&key_hex, &mut key)
        .map_err(|_| anyhow!("MFA_ENCRYPTION_KEY must be valid hexadecimal"))?;
    
    Ok(key)
}

/// Encrypt MFA secret using AES-256-GCM
pub fn encrypt_mfa_secret(secret: &str) -> Result<String> {
    let key_bytes = get_encryption_key()?;
    let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow!("Failed to create encryption key"))?;
    let sealing_key = LessSafeKey::new(unbound_key);
    
    // Generate random nonce
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow!("Failed to generate nonce"))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    
    // Encrypt the secret
    let mut in_out = secret.as_bytes().to_vec();
    sealing_key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Encryption failed"))?;
    
    // Combine nonce + ciphertext and encode as hex
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&in_out);
    Ok(hex::encode(result))
}

/// Decrypt MFA secret using AES-256-GCM
pub fn decrypt_mfa_secret(encrypted_hex: &str) -> Result<SecretString> {
    let key_bytes = get_encryption_key()?;
    let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow!("Failed to create decryption key"))?;
    let opening_key = LessSafeKey::new(unbound_key);
    
    // Decode from hex
    let encrypted_data = hex::decode(encrypted_hex)
        .map_err(|_| anyhow!("Invalid encrypted data format"))?;
    
    if encrypted_data.len() < 12 {
        return Err(anyhow!("Encrypted data too short"));
    }
    
    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| anyhow!("Invalid nonce"))?;
    
    // Decrypt
    let mut in_out = ciphertext.to_vec();
    let plaintext = opening_key.open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Decryption failed"))?;
    
    let secret = String::from_utf8(plaintext.to_vec())
        .map_err(|_| anyhow!("Invalid UTF-8 in decrypted secret"))?;
    
    Ok(SecretString::new(secret))
}

/// Generate a cryptographically secure random string for TOTP secret
/// Uses 160 bits of entropy (recommended minimum for TOTP secrets)
pub fn generate_totp_secret() -> SecretString {
    let mut secret_bytes = [0u8; 20]; // 20 bytes = 160 bits of entropy
    rand::thread_rng().fill_bytes(&mut secret_bytes);
    let secret = base32::encode(base32::Alphabet::RFC4648 { padding: true }, &secret_bytes);
    SecretString::new(secret)
}

/// Generate backup codes for MFA recovery - async version for performance
pub async fn generate_backup_codes_async() -> (Vec<String>, Vec<String>) {
    use tokio::task;
    
    let mut plaintext_codes = Vec::new();
    let mut hash_futures = Vec::new();
    
    // Generate all codes first
    for _ in 0..8 {
        let code: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>()
            .to_uppercase();
        
        let code_clone = code.clone();
        plaintext_codes.push(code);
        
        // Create async hash task to avoid blocking
        let hash_future = task::spawn_blocking(move || {
            bcrypt_hash(&code_clone, DEFAULT_COST)
                .expect("Failed to hash backup code")
        });
        hash_futures.push(hash_future);
    }
    
    // Wait for all hashing to complete
    let mut hashed_codes = Vec::new();
    for future in hash_futures {
        let hash = future.await.expect("Hash task failed");
        hashed_codes.push(hash);
    }
    
    (plaintext_codes, hashed_codes)
}

/// Generate QR code as SVG string
pub fn generate_qr_code(secret: &str, user_email: &str, service_name: &str) -> Result<String> {
    // Create TOTP URL for authenticator apps
    let totp_url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        service_name, user_email, secret, service_name
    );
    
    let code = QrCode::new(&totp_url)
        .map_err(|e| anyhow!("Failed to generate QR code: {}", e))?;
    
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(200, 200)
        .build();
    
    // Convert SVG to base64 data URL for frontend
    let base64_svg = general_purpose::STANDARD.encode(svg);
    Ok(format!("data:image/svg+xml;base64,{}", base64_svg))
}

/// Verify TOTP token with timing-attack protection and clock drift tolerance
/// Uses SHA1 algorithm for maximum compatibility with authenticator apps
pub fn verify_totp_token(secret: &str, token: &str) -> bool {
    let secret_bytes = match Secret::Encoded(secret.to_string()).to_bytes() {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    let totp = match TOTP::new(
        TotpAlgorithm::SHA1,  // SHA1 for compatibility with most authenticator apps
        6,                    // 6-digit codes (industry standard)
        1,                    // 1 step = 30 seconds
        30,                   // 30-second window (industry standard)
        secret_bytes,
    ) {
        Ok(totp) => totp,
        Err(_) => return false,
    };
    
    // TOTP verification with ±30 second tolerance for clock drift
    totp.check_current(token).unwrap_or(false) ||
    totp.check(token, chrono::Utc::now().timestamp() as u64 - 30) ||
    totp.check(token, chrono::Utc::now().timestamp() as u64 + 30)
}

/// Verify backup code and mark it as used
pub async fn verify_backup_code(
    user_uuid: &Uuid,
    provided_code: &str,
    conn: &mut DbConnection,
) -> Result<MfaVerificationResult> {
    let user = repository::get_user_by_uuid(user_uuid, conn)
        .map_err(|_| anyhow!("User not found"))?;

    // Get backup codes
    let backup_codes = user.mfa_backup_codes
        .as_ref()
        .and_then(|codes| codes.as_array())
        .ok_or_else(|| anyhow!("No backup codes found"))?;

    let mut remaining_codes = Vec::new();
    let mut code_found = false;
    let mut used_code = None;

    // Check each backup code
    for code_value in backup_codes {
        if let Some(hashed_code) = code_value.as_str() {
            if !code_found && bcrypt_verify(provided_code, hashed_code).unwrap_or(false) {
                code_found = true;
                used_code = Some(provided_code.to_string());
                // Don't add the used code to remaining_codes
            } else {
                remaining_codes.push(code_value.clone());
            }
        }
    }

    if !code_found {
        return Ok(MfaVerificationResult {
            is_valid: false,
            backup_code_used: None,
            requires_backup_code_regeneration: false,
        });
    }

    // Check if we're running low on backup codes before moving remaining_codes
    let requires_regeneration = remaining_codes.len() <= 2;

    // Update backup codes in database (remove the used one)
    let updated_codes = serde_json::Value::Array(remaining_codes);
    let mfa_update = crate::models::UserMfaUpdate {
        mfa_enabled: None,
        mfa_secret: None,
        mfa_backup_codes: Some(updated_codes),
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    repository::update_user_mfa(user_uuid, mfa_update, conn)
        .map_err(|_| anyhow!("Failed to update backup codes"))?;

    Ok(MfaVerificationResult {
        is_valid: true,
        backup_code_used: used_code,
        requires_backup_code_regeneration: requires_regeneration,
    })
}

/// Comprehensive MFA verification (TOTP or backup code)
pub async fn verify_mfa_token(
    user_uuid: &Uuid,
    token: &str,
    conn: &mut DbConnection,
) -> Result<MfaVerificationResult> {
    let user = repository::get_user_by_uuid(user_uuid, conn)
        .map_err(|_| anyhow!("User not found"))?;

    if !user.mfa_enabled {
        return Err(anyhow!("MFA is not enabled for this user"));
    }

    // First try TOTP verification
    if let Some(ref encrypted_secret) = user.mfa_secret {
        let secret = decrypt_mfa_secret(encrypted_secret)?;
        if verify_totp_token(secret.as_str(), token) {
            return Ok(MfaVerificationResult {
                is_valid: true,
                backup_code_used: None,
                requires_backup_code_regeneration: false,
            });
        }
    }

    // If TOTP fails, try backup code verification
    verify_backup_code(user_uuid, token, conn).await
}

/// Check if MFA should be required for a user based on OWASP recommendations
pub fn should_require_mfa(user_role: &UserRole) -> bool {
    match user_role {
        UserRole::Admin => true,      // OWASP: Always require for admin users
        UserRole::Technician => true, // High privilege users
        UserRole::User => false,      // Could be made configurable via env var
    }
}

/// Check if user has MFA enabled and enforce policy
pub async fn validate_mfa_policy(user: &User) -> Result<()> {
    if should_require_mfa(&user.role) && !user.mfa_enabled {
        return Err(anyhow!(
            "MFA is required for {} users. Please enable MFA on your account.", 
            match user.role {
                UserRole::Admin => "administrator",
                UserRole::Technician => "technician", 
                UserRole::User => "user",
            }
        ));
    }
    Ok(())
}

/// Check if user has MFA enabled
pub fn user_has_mfa_enabled(user: &User) -> bool {
    user.mfa_enabled && user.mfa_secret.is_some()
}

/// Log security events for MFA attempts
pub async fn log_mfa_attempt(
    user_uuid: &Uuid,
    success: bool,
    attempt_type: &str,
    request: &actix_web::HttpRequest,
) {
    let user_agent = request.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");
    
    let ip_address = request.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    
    if success {
        tracing::info!(
            "Successful MFA {} for user {} from IP {} using {}",
            attempt_type, user_uuid, ip_address, user_agent
        );
    } else {
        tracing::warn!(
            "Failed MFA {} for user {} from IP {} using {}",
            attempt_type, user_uuid, ip_address, user_agent
        );
    }
}

/// MFA rate limiting check (placeholder for future implementation)
pub async fn check_mfa_rate_limit(user_uuid: &Uuid) -> bool {
    // TODO: Implement rate limiting using Redis or database
    tracing::info!("MFA attempt for user {} (rate limiting not yet implemented)", user_uuid);
    true
} 