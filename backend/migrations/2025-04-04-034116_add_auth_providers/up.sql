-- Your SQL goes here

-- Create auth providers table
CREATE TABLE auth_providers (
    id SERIAL PRIMARY KEY,
    provider_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create auth provider configurations table
CREATE TABLE auth_provider_configs (
    id SERIAL PRIMARY KEY,
    auth_provider_id INTEGER NOT NULL REFERENCES auth_providers(id) ON DELETE CASCADE,
    config_key VARCHAR(255) NOT NULL,
    config_value TEXT NOT NULL,
    is_secret BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(auth_provider_id, config_key)
);

-- Seed local authentication provider (always available)
INSERT INTO auth_providers (provider_type, name, enabled, is_default, created_at, updated_at)
VALUES ('local', 'Local Authentication', TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- Create index on provider_type for faster lookups
CREATE INDEX idx_auth_providers_type ON auth_providers(provider_type);

-- Create index on auth_provider_id for faster joins
CREATE INDEX idx_auth_provider_configs_provider_id ON auth_provider_configs(auth_provider_id);
