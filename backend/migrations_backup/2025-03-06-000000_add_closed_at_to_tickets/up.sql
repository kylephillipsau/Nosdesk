-- Add closed_at timestamp column to tickets table
ALTER TABLE tickets ADD COLUMN closed_at TIMESTAMP;

-- Update existing closed tickets to set closed_at to modified timestamp
UPDATE tickets SET closed_at = modified WHERE status = 'closed'; 