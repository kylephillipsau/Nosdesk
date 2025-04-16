-- This file should undo anything in `up.sql`

-- Drop tables in reverse order of creation
DROP TABLE IF EXISTS documentation_page_relationships;
DROP TABLE IF EXISTS documentation_pages;
DROP TABLE IF EXISTS documentation_categories;
DROP TYPE IF EXISTS documentation_status;
