-- Add name column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS name VARCHAR(100);

-- Set default name from email (first part before @)
UPDATE users SET name = split_part(email, '@', 1) WHERE name IS NULL;
