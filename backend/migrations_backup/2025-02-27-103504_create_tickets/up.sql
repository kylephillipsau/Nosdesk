-- Your SQL goes here

-- Create ticket status enum
CREATE TYPE ticket_status AS ENUM ('open', 'in-progress', 'closed');

-- Create ticket priority enum
CREATE TYPE ticket_priority AS ENUM ('low', 'medium', 'high');

-- Create tickets table
CREATE TABLE tickets (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    status ticket_status NOT NULL DEFAULT 'open',
    priority ticket_priority NOT NULL DEFAULT 'medium',
    created DATE NOT NULL DEFAULT CURRENT_DATE,
    modified DATE NOT NULL DEFAULT CURRENT_DATE,
    assignee VARCHAR(255),
    requester VARCHAR(255) NOT NULL
);

-- Create devices table
CREATE TABLE devices (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    serial_number VARCHAR(255) NOT NULL,
    model VARCHAR(255) NOT NULL,
    warranty_status VARCHAR(50) NOT NULL,
    ticket_id INTEGER REFERENCES tickets(id) ON DELETE CASCADE
);

-- Create notes and comments table
CREATE TABLE notes (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    author VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ticket_id INTEGER REFERENCES tickets(id) ON DELETE CASCADE
);

-- Create attachments table
CREATE TABLE attachments (
    id SERIAL PRIMARY KEY,
    url VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    note_id INTEGER REFERENCES notes(id) ON DELETE CASCADE
);

-- Create article content table
CREATE TABLE article_contents (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    ticket_id INTEGER REFERENCES tickets(id) ON DELETE CASCADE,
    CONSTRAINT unique_ticket_article UNIQUE (ticket_id)
);
