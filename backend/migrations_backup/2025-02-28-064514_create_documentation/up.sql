-- Your SQL goes here

-- Create documentation_status enum type
CREATE TYPE documentation_status AS ENUM ('draft', 'published', 'archived');

-- Create documentation_categories table
CREATE TABLE documentation_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(50),
    parent_id INTEGER REFERENCES documentation_categories(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create documentation_pages table
CREATE TABLE documentation_pages (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(255) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    content TEXT NOT NULL,
    category_id INTEGER REFERENCES documentation_categories(id),
    author VARCHAR(255) NOT NULL,
    status documentation_status NOT NULL DEFAULT 'draft',
    icon VARCHAR(50),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create documentation_page_relationships table for hierarchical structure
CREATE TABLE documentation_page_relationships (
    parent_id INTEGER NOT NULL REFERENCES documentation_pages(id),
    child_id INTEGER NOT NULL REFERENCES documentation_pages(id),
    display_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (parent_id, child_id)
);
