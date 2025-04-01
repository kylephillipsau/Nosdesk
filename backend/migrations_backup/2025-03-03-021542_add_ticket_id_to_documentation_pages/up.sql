-- Your SQL goes here

-- Add ticket_id column to documentation_pages table
ALTER TABLE documentation_pages ADD COLUMN ticket_id INTEGER;

-- Add foreign key constraint
ALTER TABLE documentation_pages 
ADD CONSTRAINT fk_documentation_pages_ticket 
FOREIGN KEY (ticket_id) 
REFERENCES tickets(id) 
ON DELETE SET NULL;

-- Create an index for better performance
CREATE INDEX idx_documentation_pages_ticket_id ON documentation_pages(ticket_id);
