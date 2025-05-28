-- Create junction table for many-to-many relationship between tickets and devices
CREATE TABLE ticket_devices (
    ticket_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (ticket_id, device_id)
);

-- Add foreign key constraints
ALTER TABLE ticket_devices
    ADD CONSTRAINT ticket_devices_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;

ALTER TABLE ticket_devices
    ADD CONSTRAINT ticket_devices_device_id_fkey
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE;

-- Add indexes for better query performance
CREATE INDEX idx_ticket_devices_ticket_id ON ticket_devices(ticket_id);
CREATE INDEX idx_ticket_devices_device_id ON ticket_devices(device_id);

-- Remove the old ticket_id column from devices table if it still exists
-- (This should have been done in the previous migration, but we'll ensure it's gone)
ALTER TABLE devices DROP COLUMN IF EXISTS ticket_id;
