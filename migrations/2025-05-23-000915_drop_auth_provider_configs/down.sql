-- Recreate auth_provider_configs table
CREATE TABLE auth_provider_configs (
    id SERIAL PRIMARY KEY,
    auth_provider_id INTEGER NOT NULL REFERENCES auth_providers(id) ON DELETE CASCADE,
    config_key VARCHAR(255) NOT NULL,
    config_value TEXT NOT NULL,
    is_secret BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
); 