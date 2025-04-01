-- This file should undo anything in `up.sql`

-- Drop the index
DROP INDEX IF EXISTS idx_documentation_pages_ticket_id;

-- Drop the foreign key constraint
ALTER TABLE documentation_pages DROP CONSTRAINT IF EXISTS fk_documentation_pages_ticket;

-- Drop the ticket_id column
ALTER TABLE documentation_pages DROP COLUMN IF EXISTS ticket_id;
