-- This file should undo anything in `up.sql`

-- Remove the index first
DROP INDEX IF EXISTS idx_users_microsoft_uuid;

-- Remove the microsoft_uuid column
ALTER TABLE users DROP COLUMN IF EXISTS microsoft_uuid;
