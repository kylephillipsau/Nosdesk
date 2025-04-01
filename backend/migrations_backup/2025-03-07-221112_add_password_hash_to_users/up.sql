-- Your SQL goes here

-- Add password_hash column to users table
ALTER TABLE users ADD COLUMN password_hash VARCHAR(255) NOT NULL DEFAULT '';

-- Update existing users with a default password hash (this is just a placeholder, users should reset their passwords)
-- The default hash is for the password 'changeme'
UPDATE users SET password_hash = '$2b$12$K8HbN0CXgIYG5jvwGVT2B.4CkQIBRzwz6OzMQrL4d0eGBOJJUEpKO';
