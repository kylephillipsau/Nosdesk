use uuid::Uuid;

/// Generic rate limiting utility using Redis
/// This provides a reusable rate limiting implementation following DRY principles
pub struct RateLimiter;

#[derive(Debug)]
pub enum RateLimitError {
    RedisError(String),
    ConnectionFailed,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RedisError(msg) => write!(f, "Redis error: {}", msg),
            Self::ConnectionFailed => write!(f, "Failed to connect to Redis"),
        }
    }
}

impl std::error::Error for RateLimitError {}

impl RateLimiter {
    /// Check if a rate limit has been exceeded
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `key` - Unique key for this rate limit (e.g., "mfa_attempts:user_uuid")
    /// * `max_attempts` - Maximum number of attempts allowed
    /// * `window_seconds` - Time window in seconds
    ///
    /// # Returns
    /// * `Ok(true)` - Request is allowed (under limit)
    /// * `Ok(false)` - Rate limit exceeded
    /// * `Err(_)` - Redis connection or other error
    pub async fn check_rate_limit(
        redis_url: &str,
        key: &str,
        max_attempts: u32,
        window_seconds: u64,
    ) -> Result<bool, RateLimitError> {
        // Get the current count
        let current_count = Self::get_attempt_count(redis_url, key).await?;

        // Check if under limit
        if current_count < max_attempts {
            // Increment the counter with TTL
            Self::increment_attempt(redis_url, key, window_seconds).await?;
            Ok(true)
        } else {
            // Rate limit exceeded
            tracing::warn!("Rate limit exceeded for key: {} ({}/{})", key, current_count, max_attempts);
            Ok(false)
        }
    }

    /// Get the current attempt count for a key
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL
    /// * `key` - Unique key for this rate limit
    ///
    /// # Returns
    /// Current count (0 if key doesn't exist)
    pub async fn get_attempt_count(redis_url: &str, key: &str) -> Result<u32, RateLimitError> {
        use redis::AsyncCommands;

        let client = redis::Client::open(redis_url)
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let mut con = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| RateLimitError::ConnectionFailed)?;

        let count: Option<u32> = con
            .get(key)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        Ok(count.unwrap_or(0))
    }

    /// Increment the attempt counter for a key with automatic expiry
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL
    /// * `key` - Unique key for this rate limit
    /// * `ttl_seconds` - Time to live in seconds (auto-expire)
    pub async fn increment_attempt(
        redis_url: &str,
        key: &str,
        ttl_seconds: u64,
    ) -> Result<(), RateLimitError> {
        use redis::AsyncCommands;

        let client = redis::Client::open(redis_url)
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        let mut con = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| RateLimitError::ConnectionFailed)?;

        // Use a Lua script for atomic increment + expire
        // This ensures the TTL is set atomically with the increment
        let script = r#"
            local current = redis.call('INCR', KEYS[1])
            if current == 1 then
                redis.call('EXPIRE', KEYS[1], ARGV[1])
            end
            return current
        "#;

        redis::Script::new(script)
            .key(key)
            .arg(ttl_seconds)
            .invoke_async::<_, ()>(&mut con)
            .await
            .map_err(|e| RateLimitError::RedisError(e.to_string()))?;

        Ok(())
    }


    /// Generate a standardized rate limit key for MFA attempts
    ///
    /// # Arguments
    /// * `user_uuid` - User's UUID
    ///
    /// # Returns
    /// Formatted Redis key for MFA rate limiting
    pub fn mfa_attempt_key(user_uuid: &Uuid) -> String {
        format!("mfa_attempts:{}", user_uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfa_attempt_key_format() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = RateLimiter::mfa_attempt_key(&uuid);
        assert_eq!(key, "mfa_attempts:550e8400-e29b-41d4-a716-446655440000");
    }

    // Note: Integration tests requiring Redis would go in tests/ directory
}
