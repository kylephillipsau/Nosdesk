-- This file should undo anything in `up.sql`

-- Drop foreign key constraints first
ALTER TABLE article_contents DROP CONSTRAINT IF EXISTS article_contents_ticket_id_fkey;
ALTER TABLE attachments DROP CONSTRAINT IF EXISTS attachments_comment_id_fkey;
ALTER TABLE comments DROP CONSTRAINT IF EXISTS comments_ticket_id_fkey;
ALTER TABLE comments DROP CONSTRAINT IF EXISTS comments_user_uuid_fkey;
ALTER TABLE devices DROP CONSTRAINT IF EXISTS devices_ticket_id_fkey;
ALTER TABLE documentation_pages DROP CONSTRAINT IF EXISTS documentation_pages_parent_id_fkey;
ALTER TABLE documentation_pages DROP CONSTRAINT IF EXISTS fk_documentation_pages_ticket;
ALTER TABLE linked_tickets DROP CONSTRAINT IF EXISTS linked_tickets_linked_ticket_id_fkey;
ALTER TABLE linked_tickets DROP CONSTRAINT IF EXISTS linked_tickets_ticket_id_fkey;
ALTER TABLE project_tickets DROP CONSTRAINT IF EXISTS project_tickets_project_id_fkey;
ALTER TABLE project_tickets DROP CONSTRAINT IF EXISTS project_tickets_ticket_id_fkey;

-- Drop unique constraints
ALTER TABLE article_contents DROP CONSTRAINT IF EXISTS unique_ticket_article;

-- Drop tables
DROP TABLE IF EXISTS project_tickets;
DROP TABLE IF EXISTS linked_tickets;
DROP TABLE IF EXISTS article_contents;
DROP TABLE IF EXISTS attachments;
DROP TABLE IF EXISTS comments;
DROP TABLE IF EXISTS devices;
DROP TABLE IF EXISTS documentation_pages;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS tickets;
DROP TABLE IF EXISTS users;

-- Drop custom types
DROP TYPE IF EXISTS documentation_status;
DROP TYPE IF EXISTS project_status;
DROP TYPE IF EXISTS ticket_priority;
DROP TYPE IF EXISTS ticket_status;
DROP TYPE IF EXISTS user_role;
