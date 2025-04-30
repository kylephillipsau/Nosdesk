-- This file should undo anything in `up.sql`

-- Convert content column in documentation_pages back from BYTEA to TEXT
ALTER TABLE documentation_pages 
  ALTER COLUMN content TYPE TEXT USING convert_from(content, 'UTF8');

-- Convert content column in article_contents back from BYTEA to TEXT
ALTER TABLE article_contents 
  ALTER COLUMN content TYPE TEXT USING convert_from(content, 'UTF8');
