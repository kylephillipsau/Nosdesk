-- Revert changes made in up.sql

-- Drop the columns added to the users table
ALTER TABLE users DROP COLUMN IF EXISTS pronouns;
ALTER TABLE users DROP COLUMN IF EXISTS avatar_url;
ALTER TABLE users DROP COLUMN IF EXISTS banner_url; 