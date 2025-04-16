-- This file should undo anything in `up.sql`

-- Drop the index first
DROP INDEX IF EXISTS idx_documentation_pages_display_order;

-- Remove the display_order column
ALTER TABLE documentation_pages DROP COLUMN IF EXISTS display_order;
