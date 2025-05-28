-- This file should undo anything in `up.sql`

-- Remove avatar_thumb field from users table
ALTER TABLE users DROP COLUMN avatar_thumb;
