-- Your SQL goes here

-- Create a new table to store external auth identities for users
CREATE TABLE user_auth_identities (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    auth_provider_id INTEGER NOT NULL REFERENCES auth_providers(id) ON DELETE CASCADE,
    provider_user_id VARCHAR(255) NOT NULL, -- External ID from the provider (e.g., Microsoft ID)
    email VARCHAR(255), -- Email associated with this identity (may differ from user's primary email)
    identity_data JSONB, -- Additional identity data from the provider (profile info, etc.)
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(auth_provider_id, provider_user_id) -- Ensure each external identity is linked to only one user
);

-- Create indexes for fast lookups
CREATE INDEX idx_user_auth_identities_user_id ON user_auth_identities(user_id);
CREATE INDEX idx_user_auth_identities_auth_provider_id ON user_auth_identities(auth_provider_id);
CREATE INDEX idx_user_auth_identities_email ON user_auth_identities(email);

-- Populate the table with existing OAuth users
-- This will create identity records for users who were already created through OAuth
-- We'll use a CTE to handle this more elegantly
WITH oauth_users AS (
    -- Find users without password hash (likely created through OAuth)
    -- This is an assumption, you might need to adjust this logic
    SELECT id FROM users 
    WHERE length(password_hash) = 0 OR password_hash IS NULL
)
INSERT INTO user_auth_identities (user_id, auth_provider_id, provider_user_id, email, created_at, updated_at)
SELECT 
    u.id, 
    (SELECT id FROM auth_providers WHERE provider_type = 'microsoft' LIMIT 1), -- Assuming Microsoft provider
    u.uuid, -- Use UUID as a fallback provider_user_id
    u.email,
    NOW(),
    NOW()
FROM users u
JOIN oauth_users o ON u.id = o.id
-- Only do this if we have a Microsoft provider configured
WHERE EXISTS (SELECT 1 FROM auth_providers WHERE provider_type = 'microsoft');

-- Add a comment to explain the table
COMMENT ON TABLE user_auth_identities IS 'Links users to their external authentication identities from various providers';
