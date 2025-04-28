-- This file should undo anything in `up.sql`

-- Add password_hash column back to users table
ALTER TABLE users ADD COLUMN password_hash BYTEA;

-- Copy password hashes from user_auth_identities back to users table
UPDATE users u
SET password_hash = uai.password_hash
FROM user_auth_identities uai, auth_providers ap
WHERE u.id = uai.user_id
  AND uai.auth_provider_id = ap.id
  AND ap.provider_type = 'local'
  AND uai.password_hash IS NOT NULL;

-- Make password_hash NOT NULL in users table
ALTER TABLE users ALTER COLUMN password_hash SET NOT NULL;

-- We keep the password_hash column in user_auth_identities for consistency
