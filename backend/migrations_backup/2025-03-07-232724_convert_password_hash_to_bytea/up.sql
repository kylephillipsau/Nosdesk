-- Your SQL goes here

-- Convert password_hash column from VARCHAR to BYTEA
-- First, update any existing records with missing prefix
UPDATE users 
SET password_hash = CONCAT('$2b$12$', SUBSTRING(password_hash FROM 3))
WHERE password_hash LIKE 'b$%';

-- Remove the default constraint
ALTER TABLE users ALTER COLUMN password_hash DROP DEFAULT;

-- Convert the column to BYTEA
ALTER TABLE users ALTER COLUMN password_hash TYPE BYTEA USING password_hash::bytea;

-- Set a new default if needed
ALTER TABLE users ALTER COLUMN password_hash SET DEFAULT '\x'::bytea;
