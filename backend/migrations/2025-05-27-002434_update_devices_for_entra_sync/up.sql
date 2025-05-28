-- Your SQL goes here

-- Update devices table for Microsoft Entra/Intune device synchronization

-- First, drop the foreign key constraint for ticket_id
ALTER TABLE devices DROP CONSTRAINT IF EXISTS devices_ticket_id_fkey;

-- Remove the ticket_id column since devices can be associated with multiple tickets
ALTER TABLE devices DROP COLUMN IF EXISTS ticket_id;

-- Add new columns for Microsoft Entra/Intune integration
ALTER TABLE devices ADD COLUMN manufacturer VARCHAR(255);
ALTER TABLE devices ADD COLUMN primary_user_uuid VARCHAR(36);
ALTER TABLE devices ADD COLUMN intune_device_id VARCHAR(255);
ALTER TABLE devices ADD COLUMN entra_device_id VARCHAR(255);
ALTER TABLE devices ADD COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE devices ADD COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Add foreign key constraint for primary_user_uuid
ALTER TABLE devices 
    ADD CONSTRAINT devices_primary_user_uuid_fkey 
    FOREIGN KEY (primary_user_uuid) REFERENCES users(uuid) ON DELETE SET NULL;

-- Add unique constraints for the Microsoft IDs to prevent duplicates
ALTER TABLE devices ADD CONSTRAINT devices_intune_device_id_unique UNIQUE (intune_device_id);
ALTER TABLE devices ADD CONSTRAINT devices_entra_device_id_unique UNIQUE (entra_device_id);

-- Add indexes for better query performance
CREATE INDEX idx_devices_manufacturer ON devices(manufacturer);
CREATE INDEX idx_devices_primary_user_uuid ON devices(primary_user_uuid);
CREATE INDEX idx_devices_intune_device_id ON devices(intune_device_id);
CREATE INDEX idx_devices_entra_device_id ON devices(entra_device_id);
CREATE INDEX idx_devices_created_at ON devices(created_at);
CREATE INDEX idx_devices_updated_at ON devices(updated_at);
