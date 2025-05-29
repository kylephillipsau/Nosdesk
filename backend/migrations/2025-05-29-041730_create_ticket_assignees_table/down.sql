-- This file should undo anything in `up.sql`

-- Drop the junction table and its constraints/indexes
DROP INDEX IF EXISTS idx_ticket_assignees_user_uuid;
DROP INDEX IF EXISTS idx_ticket_assignees_ticket_id;
DROP TABLE IF EXISTS ticket_assignees;
