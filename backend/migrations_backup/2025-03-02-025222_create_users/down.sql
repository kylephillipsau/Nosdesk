-- This file should undo anything in `up.sql`

-- Remove foreign key constraints from tickets table
ALTER TABLE tickets DROP CONSTRAINT IF EXISTS fk_tickets_assignee;
ALTER TABLE tickets DROP CONSTRAINT IF EXISTS fk_tickets_requester;

-- Drop indexes
DROP INDEX IF EXISTS idx_users_uuid;
DROP INDEX IF EXISTS idx_users_email;

-- Drop the users table
DROP TABLE IF EXISTS users;

-- Drop the user_role enum type
DROP TYPE IF EXISTS user_role;
