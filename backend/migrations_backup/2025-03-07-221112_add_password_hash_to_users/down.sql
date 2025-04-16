-- This file should undo anything in `up.sql`

-- Remove password_hash column from users table
ALTER TABLE users DROP COLUMN password_hash;
