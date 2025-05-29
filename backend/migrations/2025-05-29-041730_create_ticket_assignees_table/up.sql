-- Your SQL goes here

-- Create junction table for many-to-many relationship between tickets and assignees
CREATE TABLE ticket_assignees (
    ticket_id INTEGER NOT NULL,
    user_uuid VARCHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (ticket_id, user_uuid)
);

-- Add foreign key constraints
ALTER TABLE ticket_assignees
    ADD CONSTRAINT ticket_assignees_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;

ALTER TABLE ticket_assignees
    ADD CONSTRAINT ticket_assignees_user_uuid_fkey
    FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE;

-- Add indexes for better query performance
CREATE INDEX idx_ticket_assignees_ticket_id ON ticket_assignees(ticket_id);
CREATE INDEX idx_ticket_assignees_user_uuid ON ticket_assignees(user_uuid);

-- Migrate existing assignee data to the new table
-- This will handle the current inconsistent data by trying to match UUIDs first
INSERT INTO ticket_assignees (ticket_id, user_uuid, created_at)
SELECT 
    t.id,
    u.uuid,
    CURRENT_TIMESTAMP
FROM tickets t
INNER JOIN users u ON (
    t.assignee = u.uuid OR 
    t.assignee = u.email OR 
    t.assignee = u.name
)
WHERE t.assignee IS NOT NULL AND t.assignee != '';

-- Clear the old assignee column since we're moving to the junction table
UPDATE tickets SET assignee = NULL;
