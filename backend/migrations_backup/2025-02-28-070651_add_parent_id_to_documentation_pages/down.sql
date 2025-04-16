-- This file should undo anything in `up.sql`

-- Drop the index first
DROP INDEX IF EXISTS idx_documentation_pages_parent_id;

-- Remove the parent_id column
ALTER TABLE documentation_pages
DROP COLUMN IF EXISTS parent_id;
