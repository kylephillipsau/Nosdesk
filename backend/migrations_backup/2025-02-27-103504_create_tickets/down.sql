-- This file should undo anything in `up.sql`

-- Drop tables in reverse order of creation (to handle foreign key constraints)
DROP TABLE IF EXISTS article_contents;
DROP TABLE IF EXISTS attachments;
DROP TABLE IF EXISTS notes;
DROP TABLE IF EXISTS devices;
DROP TABLE IF EXISTS tickets;

-- Drop enum types
DROP TYPE IF EXISTS ticket_priority;
DROP TYPE IF EXISTS ticket_status;
