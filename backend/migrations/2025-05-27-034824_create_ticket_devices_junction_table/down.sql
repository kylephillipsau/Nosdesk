-- This file should undo anything in `up.sql`

-- Drop indexes
DROP INDEX IF EXISTS idx_ticket_devices_device_id;
DROP INDEX IF EXISTS idx_ticket_devices_ticket_id;

-- Drop foreign key constraints
ALTER TABLE ticket_devices DROP CONSTRAINT IF EXISTS ticket_devices_device_id_fkey;
ALTER TABLE ticket_devices DROP CONSTRAINT IF EXISTS ticket_devices_ticket_id_fkey;

-- Drop the junction table
DROP TABLE IF EXISTS ticket_devices;

-- Add back the ticket_id column to devices table (for rollback compatibility)
ALTER TABLE devices ADD COLUMN ticket_id INTEGER;

-- Add back the foreign key constraint for ticket_id
ALTER TABLE devices 
    ADD CONSTRAINT devices_ticket_id_fkey 
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;
