-- This file should undo anything in `up.sql`

-- Drop the index
DROP INDEX IF EXISTS idx_notes_is_comment;

-- Drop the is_comment column
ALTER TABLE notes DROP COLUMN IF EXISTS is_comment;
