-- Your SQL goes here

-- Convert content column in documentation_pages from TEXT to BYTEA
ALTER TABLE documentation_pages 
  ALTER COLUMN content TYPE BYTEA USING content::bytea;

-- Convert content column in article_contents from TEXT to BYTEA
ALTER TABLE article_contents 
  ALTER COLUMN content TYPE BYTEA USING content::bytea;
