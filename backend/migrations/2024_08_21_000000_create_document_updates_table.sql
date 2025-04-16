-- Create the document_updates table for collaborative editing
CREATE TABLE IF NOT EXISTS document_updates (
    id SERIAL PRIMARY KEY,
    document_id VARCHAR(255) NOT NULL,
    update_data BYTEA NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add indexes for faster querying
CREATE INDEX IF NOT EXISTS document_updates_document_id_idx ON document_updates(document_id);
CREATE INDEX IF NOT EXISTS document_updates_created_at_idx ON document_updates(created_at); 