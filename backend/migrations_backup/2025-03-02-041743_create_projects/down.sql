-- This file should undo anything in `up.sql`

-- Drop the join table first (because it references projects)
DROP TABLE IF EXISTS project_tickets;

-- Drop the projects table
DROP TABLE IF EXISTS projects;

-- Drop the enum type
DROP TYPE IF EXISTS project_status;
