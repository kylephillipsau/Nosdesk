-- Your SQL goes here

CREATE TABLE sync_history (
    id SERIAL PRIMARY KEY,
    session_id VARCHAR NOT NULL UNIQUE,
    sync_type VARCHAR NOT NULL,
    entity VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    message TEXT NOT NULL,
    current_count INTEGER NOT NULL DEFAULT 0,
    total_count INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    can_cancel BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sync_history_session_id ON sync_history(session_id);
CREATE INDEX idx_sync_history_status ON sync_history(status);
CREATE INDEX idx_sync_history_sync_type ON sync_history(sync_type);
CREATE INDEX idx_sync_history_completed_at ON sync_history(completed_at);
