-- Your SQL goes here

-- Add microsoft_uuid column to users table
ALTER TABLE users ADD COLUMN microsoft_uuid VARCHAR(36);

-- Add index for performance when querying by Microsoft UUID
CREATE INDEX idx_users_microsoft_uuid ON users(microsoft_uuid);

-- Add comment to document the purpose of this column
COMMENT ON COLUMN users.microsoft_uuid IS 'Microsoft Graph/Entra UUID for users synced from Microsoft systems';
