-- Your SQL goes here

-- Add password_hash column to user_auth_identities table if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'user_auth_identities' 
        AND column_name = 'password_hash'
    ) THEN
        ALTER TABLE user_auth_identities ADD COLUMN password_hash BYTEA;
    END IF;
END $$;

-- Copy existing password hashes from users table to user_auth_identities
UPDATE user_auth_identities uai
SET password_hash = u.password_hash
FROM users u, auth_providers ap
WHERE uai.user_id = u.id
  AND uai.auth_provider_id = ap.id
  AND ap.provider_type = 'local'
  AND uai.password_hash IS NULL;

-- First make password_hash nullable, then drop it
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;
ALTER TABLE users DROP COLUMN password_hash;
