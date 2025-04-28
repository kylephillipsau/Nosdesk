-- Create local auth identities for all existing users
-- This script assumes the auth_providers table already has a 'local' provider with ID 1

-- Insert local auth identities for all users who don't already have one
WITH inserted_records AS (
    INSERT INTO user_auth_identities (
        user_id,
        auth_provider_id,
        provider_user_id,
        email,
        created_at,
        updated_at
    )
    SELECT 
        u.id,
        1, -- auth_provider_id (assuming local provider ID is 1)
        u.uuid, -- using user's UUID as provider_user_id
        u.email,
        NOW(),
        NOW()
    FROM users u
    LEFT JOIN user_auth_identities uai ON 
        u.id = uai.user_id AND 
        uai.auth_provider_id = 1
    WHERE uai.id IS NULL
    RETURNING user_id
)
SELECT COUNT(*) AS users_processed FROM inserted_records;

-- Display all auth identities for verification
SELECT 
    u.id AS user_id, 
    u.name AS user_name, 
    u.email, 
    ap.provider_type,
    uai.id AS auth_identity_id
FROM users u
JOIN user_auth_identities uai ON u.id = uai.user_id
JOIN auth_providers ap ON uai.auth_provider_id = ap.id
ORDER BY u.id, ap.provider_type; 