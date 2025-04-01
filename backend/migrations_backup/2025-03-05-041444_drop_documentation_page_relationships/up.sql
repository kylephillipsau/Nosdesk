-- Your SQL goes here

-- Drop the documentation_page_relationships table as it's no longer needed
-- The relationships are now handled by the parent_id and display_order fields in the documentation_pages table
DROP TABLE IF EXISTS documentation_page_relationships;
