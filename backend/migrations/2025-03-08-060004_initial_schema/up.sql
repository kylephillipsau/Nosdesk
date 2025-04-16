-- Your SQL goes here

-- Create custom types
CREATE TYPE documentation_status AS ENUM (
    'draft',
    'published',
    'archived'
);

CREATE TYPE project_status AS ENUM (
    'active',
    'completed',
    'archived'
);

CREATE TYPE ticket_priority AS ENUM (
    'low',
    'medium',
    'high'
);

CREATE TYPE ticket_status AS ENUM (
    'open',
    'in-progress',
    'closed'
);

CREATE TYPE user_role AS ENUM (
    'admin',
    'technician',
    'user'
);

-- Create tables
CREATE TABLE article_contents (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    ticket_id INTEGER
);

CREATE TABLE attachments (
    id SERIAL PRIMARY KEY,
    url VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    comment_id INTEGER
);

CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_uuid VARCHAR(36) NOT NULL,
    ticket_id INTEGER NOT NULL
);

CREATE TABLE devices (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    serial_number VARCHAR(255) NOT NULL,
    model VARCHAR(255) NOT NULL,
    warranty_status VARCHAR(50) NOT NULL,
    ticket_id INTEGER
);

CREATE TABLE documentation_pages (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(255) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    content TEXT NOT NULL,
    author VARCHAR(255) NOT NULL,
    status documentation_status NOT NULL DEFAULT 'draft',
    icon VARCHAR(50),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    parent_id INTEGER,
    ticket_id INTEGER,
    display_order INTEGER DEFAULT 0
);

CREATE TABLE linked_tickets (
    ticket_id INTEGER NOT NULL,
    linked_ticket_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (ticket_id, linked_ticket_id),
    CONSTRAINT no_self_link CHECK (ticket_id <> linked_ticket_id)
);

CREATE TABLE project_tickets (
    project_id INTEGER NOT NULL,
    ticket_id INTEGER NOT NULL,
    PRIMARY KEY (project_id, ticket_id)
);

CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status project_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE tickets (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    status ticket_status NOT NULL DEFAULT 'open',
    priority ticket_priority NOT NULL DEFAULT 'medium',
    created TIMESTAMP NOT NULL DEFAULT CURRENT_DATE,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    assignee VARCHAR(255),
    requester VARCHAR(255),
    closed_at TIMESTAMP
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    password_hash BYTEA NOT NULL DEFAULT '\x'::BYTEA
);

-- Create indexes
CREATE INDEX idx_documentation_pages_display_order ON documentation_pages (display_order);
CREATE INDEX idx_documentation_pages_parent_id ON documentation_pages (parent_id);
CREATE INDEX idx_documentation_pages_ticket_id ON documentation_pages (ticket_id);
CREATE INDEX idx_linked_tickets_linked_ticket_id ON linked_tickets (linked_ticket_id);
CREATE INDEX idx_linked_tickets_ticket_id ON linked_tickets (ticket_id);
CREATE INDEX idx_project_tickets_project_id ON project_tickets (project_id);
CREATE INDEX idx_project_tickets_ticket_id ON project_tickets (ticket_id);
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_uuid ON users (uuid);

-- Create foreign key constraints
ALTER TABLE article_contents
    ADD CONSTRAINT article_contents_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;

ALTER TABLE attachments
    ADD CONSTRAINT attachments_comment_id_fkey
    FOREIGN KEY (comment_id) REFERENCES comments(id);

ALTER TABLE comments
    ADD CONSTRAINT comments_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES tickets(id);

ALTER TABLE comments
    ADD CONSTRAINT comments_user_uuid_fkey
    FOREIGN KEY (user_uuid) REFERENCES users(uuid);

ALTER TABLE devices
    ADD CONSTRAINT devices_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;

ALTER TABLE documentation_pages
    ADD CONSTRAINT documentation_pages_parent_id_fkey
    FOREIGN KEY (parent_id) REFERENCES documentation_pages(id);

ALTER TABLE documentation_pages
    ADD CONSTRAINT fk_documentation_pages_ticket
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE SET NULL;

ALTER TABLE linked_tickets
    ADD CONSTRAINT linked_tickets_linked_ticket_id_fkey
    FOREIGN KEY (linked_ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;

ALTER TABLE linked_tickets
    ADD CONSTRAINT linked_tickets_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;

ALTER TABLE project_tickets
    ADD CONSTRAINT project_tickets_project_id_fkey
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE;

ALTER TABLE project_tickets
    ADD CONSTRAINT project_tickets_ticket_id_fkey
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE;

-- Add unique constraint for article_contents
ALTER TABLE article_contents
    ADD CONSTRAINT unique_ticket_article UNIQUE (ticket_id);
