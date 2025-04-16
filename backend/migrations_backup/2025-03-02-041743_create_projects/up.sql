-- Your SQL goes here

-- Create projects table
CREATE TYPE project_status AS ENUM ('active', 'completed', 'archived');

CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status project_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create a join table for tickets and projects
CREATE TABLE project_tickets (
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    ticket_id INTEGER NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    PRIMARY KEY (project_id, ticket_id)
);

-- Add indexes for better performance
CREATE INDEX idx_project_tickets_project_id ON project_tickets(project_id);
CREATE INDEX idx_project_tickets_ticket_id ON project_tickets(ticket_id);
