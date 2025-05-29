-- Create user_emails table for storing multiple email addresses per user
CREATE TABLE user_emails (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    email_type VARCHAR(50) NOT NULL DEFAULT 'primary', -- 'primary', 'alias', 'proxy', 'other'
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    source VARCHAR(50), -- 'local', 'microsoft', 'google', etc.
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_user_emails_user_id ON user_emails(user_id);
CREATE INDEX idx_user_emails_email ON user_emails(email);
CREATE INDEX idx_user_emails_is_primary ON user_emails(is_primary);
CREATE INDEX idx_user_emails_email_type ON user_emails(email_type);
CREATE INDEX idx_user_emails_source ON user_emails(source);

-- Ensure each email is unique across the system
ALTER TABLE user_emails ADD CONSTRAINT unique_email UNIQUE (email);

-- Ensure only one primary email per user
CREATE UNIQUE INDEX idx_user_emails_primary_unique ON user_emails(user_id) WHERE is_primary = TRUE;

-- Migrate existing user emails to the new table
INSERT INTO user_emails (user_id, email, email_type, is_primary, verified, source, created_at, updated_at)
SELECT 
    id as user_id,
    email,
    'primary' as email_type,
    TRUE as is_primary,
    TRUE as verified,
    'local' as source,
    created_at,
    updated_at
FROM users 
WHERE email IS NOT NULL AND email != '';

-- Add a comment explaining the table structure
COMMENT ON TABLE user_emails IS 'Stores multiple email addresses for users including primary, aliases, and proxy addresses';
COMMENT ON COLUMN user_emails.email_type IS 'Type of email: primary, alias, proxy, other';
COMMENT ON COLUMN user_emails.is_primary IS 'Whether this is the primary email address for the user';
COMMENT ON COLUMN user_emails.verified IS 'Whether the email address has been verified';
COMMENT ON COLUMN user_emails.source IS 'Source system that provided this email: local, microsoft, google, etc.'; 