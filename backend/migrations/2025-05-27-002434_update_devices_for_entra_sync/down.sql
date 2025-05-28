-- This file should undo anything in `up.sql`

-- Reverse the devices table changes for Microsoft Entra/Intune device synchronization

-- Drop indexes
DROP INDEX IF EXISTS idx_devices_updated_at;
DROP INDEX IF EXISTS idx_devices_created_at;
DROP INDEX IF EXISTS idx_devices_entra_device_id;
DROP INDEX IF EXISTS idx_devices_intune_device_id;
DROP INDEX IF EXISTS idx_devices_primary_user_uuid;
DROP INDEX IF EXISTS idx_devices_manufacturer;

-- Drop unique constraints
ALTER TABLE devices DROP CONSTRAINT IF EXISTS devices_entra_device_id_unique;
ALTER TABLE devices DROP CONSTRAINT IF EXISTS devices_intune_device_id_unique;

-- Drop foreign key constraint
ALTER TABLE devices DROP CONSTRAINT IF EXISTS devices_primary_user_uuid_fkey;

-- Remove the new columns
ALTER TABLE devices DROP COLUMN IF EXISTS updated_at;
ALTER TABLE devices DROP COLUMN IF EXISTS created_at;
ALTER TABLE devices DROP COLUMN IF EXISTS entra_device_id;
ALTER TABLE devices DROP COLUMN IF EXISTS intune_device_id;
ALTER TABLE devices DROP COLUMN IF EXISTS primary_user_uuid;
ALTER TABLE devices DROP COLUMN IF EXISTS manufacturer;

-- Add back the ticket_id column
ALTER TABLE devices ADD COLUMN ticket_id INTEGER;

-- Add back the foreign key constraint for ticket_id
ALTER TABLE devices 
    ADD CONSTRAINT devices_ticket_id_fkey 
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;
