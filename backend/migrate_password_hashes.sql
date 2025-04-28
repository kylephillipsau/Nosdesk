-- STEP 1: Add password_hash column to user_auth_identities
-- ALTER TABLE user_auth_identities ADD COLUMN password_hash BYTEA;

-- STEP 2: Copy password_hash from users to user_auth_identities where provider_type is 'local'
UPDATE user_auth_identities uai
SET password_hash = u.password_hash
FROM users u, auth_providers ap
WHERE uai.user_id = u.id
  AND uai.auth_provider_id = ap.id
  AND ap.provider_type = 'local';

-- STEP 3: Verify the migration was successful
SELECT 
    u.id AS user_id,
    u.name AS user_name,
    u.email,
    ap.provider_type,
    CASE 
        WHEN uai.password_hash IS NOT NULL THEN 'Has password'
        ELSE 'No password'
    END AS password_status
FROM users u
JOIN user_auth_identities uai ON u.id = uai.user_id
JOIN auth_providers ap ON uai.auth_provider_id = ap.id
WHERE ap.provider_type = 'local'
ORDER BY u.id;

-- STEP 4: Make the password_hash column in users nullable
-- ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- STEP 5: Set password_hash to NULL in users table
-- Uncomment this once you've verified the migration was successful
UPDATE users SET password_hash = NULL;

-- STEP 6: Next steps (to be executed later after code updates)
-- ALTER TABLE users DROP COLUMN password_hash; 