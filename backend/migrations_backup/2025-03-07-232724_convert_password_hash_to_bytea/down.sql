-- This file should undo anything in `up.sql`

-- Convert password_hash column back to VARCHAR
ALTER TABLE users ALTER COLUMN password_hash DROP DEFAULT;
ALTER TABLE users ALTER COLUMN password_hash TYPE VARCHAR(255) USING password_hash::text;
ALTER TABLE users ALTER COLUMN password_hash SET DEFAULT '';
