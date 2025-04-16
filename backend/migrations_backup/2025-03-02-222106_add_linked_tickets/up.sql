-- Your SQL goes here

-- Create a table for linked tickets
CREATE TABLE linked_tickets (
    ticket_id INT NOT NULL,
    linked_ticket_id INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (ticket_id, linked_ticket_id),
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    FOREIGN KEY (linked_ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    -- Ensure a ticket can't be linked to itself
    CONSTRAINT no_self_link CHECK (ticket_id <> linked_ticket_id)
);

-- Create an index for faster lookups
CREATE INDEX idx_linked_tickets_ticket_id ON linked_tickets(ticket_id);
CREATE INDEX idx_linked_tickets_linked_ticket_id ON linked_tickets(linked_ticket_id);
